use slipstream_client::{SlipstreamClient, SlipstreamConfig, SlipstreamError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// The `/api/rates` body recorded live on 2026-08-02 (PLAN.md section 2).
const RATES_BODY: &str = r#"{"market_rate":2.0, "multiplier":2.0, "multiplier_discount_percent":0,
 "discounted_multiplier":2.0, "submit_fee_rate":2.0,
 "slipstream_rate":4.0, "effective_rate":4.0}"#;

/// The exact `/api/transactions` 400 body recorded live (PLAN.md section 2).
const CLIENT_CODE_REQUIRED_BODY: &str = r#"{"status":"error","message":"Client codes are currently required to submit transactions. Contact us at foundation@mara.com for more information."}"#;

fn config(base_url: String, client_code: &str) -> SlipstreamConfig {
    SlipstreamConfig {
        base_url,
        fee_endpoint: "/api/rates".to_string(),
        submit_endpoint: "/api/transactions".to_string(),
        client_code: client_code.to_string(),
        request_timeout_secs: 5,
    }
}

struct NoQueryParam(&'static str);

impl wiremock::Match for NoQueryParam {
    fn matches(&self, request: &Request) -> bool {
        !request.url.query_pairs().any(|(k, _)| k == self.0)
    }
}

struct QueryParamEquals(&'static str, &'static str);

impl wiremock::Match for QueryParamEquals {
    fn matches(&self, request: &Request) -> bool {
        request
            .url
            .query_pairs()
            .any(|(k, v)| k == self.0 && v == self.1)
    }
}

#[tokio::test]
async fn rates_parses_the_recorded_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/rates"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RATES_BODY))
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), ""));
    let fee_info = client.rates().await.expect("rates should succeed");

    assert_eq!(fee_info.effective_rate_sat_vb, 4.0);
}

#[tokio::test]
async fn rates_without_client_code_omits_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/rates"))
        .and(NoQueryParam("client_code"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RATES_BODY))
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), ""));
    client
        .rates()
        .await
        .expect("request without client_code query param should match the mock");
}

#[tokio::test]
async fn rates_with_client_code_includes_it() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/rates"))
        .and(QueryParamEquals("client_code", "MYCODE123"))
        .respond_with(ResponseTemplate::new(200).set_body_string(RATES_BODY))
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), "MYCODE123"));
    client
        .rates()
        .await
        .expect("request with client_code query param should match the mock");
}

#[tokio::test]
async fn submit_tx_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/transactions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"status":"success","message":"accepted"}"#),
        )
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), "MYCODE123"));
    let result = client
        .submit_tx("deadbeef")
        .await
        .expect("submission should succeed");

    assert_eq!(result.status, "success");
    assert_eq!(result.message, "accepted");
}

#[tokio::test]
async fn submit_tx_400_client_code_required() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/transactions"))
        .respond_with(ResponseTemplate::new(400).set_body_string(CLIENT_CODE_REQUIRED_BODY))
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), "MYCODE123"));
    let err = client
        .submit_tx("deadbeef")
        .await
        .expect_err("400 client-code body should be a typed error");

    match err {
        SlipstreamError::ClientCode(message) => {
            assert_eq!(
                message,
                "Client codes are currently required to submit transactions. \
Contact us at foundation@mara.com for more information."
            );
        }
        other => panic!("expected ClientCode error, got {other:?}"),
    }
}

#[tokio::test]
async fn transport_failure_is_the_network_variant() {
    // Nothing is listening here: connection is refused immediately, no
    // wiremock server needed.
    let client = SlipstreamClient::new(&config("http://127.0.0.1:1".to_string(), ""));
    let err = client
        .rates()
        .await
        .expect_err("unreachable server should fail");

    assert!(
        matches!(err, SlipstreamError::Transport(_)),
        "expected Transport, got {err:?}"
    );
}

#[tokio::test]
async fn client_code_never_appears_in_error_output() {
    let distinctive_code = "REDACT-ME-98765-DISTINCTIVE";
    let client = SlipstreamClient::new(&config("http://127.0.0.1:1".to_string(), distinctive_code));

    let err = client
        .rates()
        .await
        .expect_err("unreachable server should fail");

    let display = err.to_string();
    let debug = format!("{err:?}");

    assert!(
        !display.contains(distinctive_code),
        "Display leaked the client code: {display}"
    );
    assert!(
        !debug.contains(distinctive_code),
        "Debug leaked the client code: {debug}"
    );
}

#[tokio::test]
async fn oversized_response_body_is_rejected() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/rates"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 65_537]))
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), "MYCODE123"));
    let err = client
        .rates()
        .await
        .expect_err("an oversized response must fail");

    match err {
        SlipstreamError::Http { status, body } => {
            assert_eq!(status, 200);
            assert_eq!(body, "response body exceeds 65536 bytes");
        }
        other => panic!("expected Http error, got {other:?}"),
    }
}

#[tokio::test]
async fn credential_is_redacted_from_parsed_response_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/transactions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"status":"error","message":"credential MYCODE123 was rejected"}"#,
            ),
        )
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), "MYCODE123"));
    let err = client
        .submit_tx("deadbeef")
        .await
        .expect_err("the response is a rejection");

    match err {
        SlipstreamError::Rejected(message) => {
            assert_eq!(message, "credential <redacted> was rejected");
        }
        other => panic!("expected Rejected error, got {other:?}"),
    }
}

#[tokio::test]
async fn credential_is_redacted_from_malformed_response_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/transactions"))
        .respond_with(ResponseTemplate::new(502).set_body_string("bad MYCODE123 response"))
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), "MYCODE123"));
    let err = client
        .submit_tx("deadbeef")
        .await
        .expect_err("the malformed response must fail");

    let display = err.to_string();
    let debug = format!("{err:?}");
    assert_eq!(
        display,
        "unexpected response (HTTP 502): could not parse submission response"
    );
    assert!(!display.contains("MYCODE123"));
    assert!(!debug.contains("MYCODE123"));
}

#[tokio::test]
async fn json_special_characters_in_credential_do_not_leak_from_malformed_response() {
    let server = MockServer::start().await;
    let client_code = "MY\"CODE\\WITH\nSPECIAL";
    let encoded_client_code = serde_json::to_string(client_code).unwrap();
    let malformed_body = format!(r#"{{"client_code":{encoded_client_code}"#);
    Mock::given(method("POST"))
        .and(path("/api/transactions"))
        .respond_with(ResponseTemplate::new(502).set_body_string(malformed_body))
        .mount(&server)
        .await;

    let client = SlipstreamClient::new(&config(server.uri(), client_code));
    let err = client
        .submit_tx("deadbeef")
        .await
        .expect_err("the malformed response must fail");

    let display = err.to_string();
    let debug = format!("{err:?}");
    assert_eq!(
        display,
        "unexpected response (HTTP 502): could not parse submission response"
    );
    assert!(!display.contains(client_code));
    assert!(!display.contains(&encoded_client_code));
    assert!(!debug.contains(client_code));
    assert!(!debug.contains(&encoded_client_code));
}
