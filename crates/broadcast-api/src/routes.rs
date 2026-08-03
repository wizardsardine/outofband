//! `AppState`, the fee cache, and all three routes: `GET /fee`,
//! `POST /broadcast`, and `GET /health`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use slipstream_client::{FeeInfo, SlipstreamClient, SlipstreamError};
use tokio::sync::RwLock;

use crate::rate_limit::{self, RateLimiter};

/// After this many consecutive failed refresh attempts, `/fee` reports
/// `stale: true` alongside the last-known value rather than silently
/// serving a number that may no longer reflect Slipstream's floor.
const STALE_AFTER_CONSECUTIVE_FAILURES: u32 = 3;
const BROADCAST_JSON_OVERHEAD_BYTES: usize = r#"{"tx_hex":""}"#.len();

#[derive(Clone)]
pub struct AppState {
    pub slipstream: Arc<SlipstreamClient>,
    pub fee_cache: FeeCache,
    pub rate_limiter: RateLimiter,
}

/// `Arc<RwLock<..>>` around the last successfully polled [`FeeInfo`] plus a
/// consecutive-failure counter. Handlers only ever read it; a background
/// task (spawned in `main`) is the only writer, so an anonymous page load
/// never fans out into a request against MARA.
#[derive(Clone)]
pub struct FeeCache {
    inner: Arc<RwLock<FeeCacheInner>>,
}

struct FeeCacheInner {
    fee: Option<FeeInfo>,
    consecutive_failures: u32,
}

impl FeeCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(FeeCacheInner {
                fee: None,
                consecutive_failures: 0,
            })),
        }
    }

    pub async fn record_success(&self, fee: FeeInfo) {
        let mut inner = self.inner.write().await;
        inner.fee = Some(fee);
        inner.consecutive_failures = 0;
    }

    pub async fn record_failure(&self) {
        let mut inner = self.inner.write().await;
        inner.consecutive_failures = inner.consecutive_failures.saturating_add(1);
    }

    async fn snapshot(&self) -> (Option<FeeInfo>, bool) {
        let inner = self.inner.read().await;
        (
            inner.fee,
            inner.consecutive_failures >= STALE_AFTER_CONSECUTIVE_FAILURES,
        )
    }
}

impl Default for FeeCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn router(state: AppState, max_payload_bytes: usize) -> Router {
    Router::new()
        .route("/fee", get(get_fee))
        .route("/broadcast", post(post_broadcast))
        .route("/health", get(get_health))
        .layer(DefaultBodyLimit::max(
            max_payload_bytes.saturating_add(BROADCAST_JSON_OVERHEAD_BYTES),
        ))
        .with_state(state)
}

#[derive(Serialize)]
struct FeeResponse {
    effective_rate_sat_vb: f64,
    age_secs: i64,
    stale: bool,
}

#[derive(Serialize)]
struct FeeUnavailableResponse {
    stale: bool,
    message: &'static str,
}

async fn get_fee(State(state): State<AppState>) -> impl IntoResponse {
    let (fee, stale) = state.fee_cache.snapshot().await;
    match fee {
        Some(fee) => {
            let age_secs = (Utc::now() - fee.fetched_at).num_seconds().max(0);
            Json(FeeResponse {
                effective_rate_sat_vb: fee.effective_rate_sat_vb,
                age_secs,
                stale,
            })
            .into_response()
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(FeeUnavailableResponse {
                stale: true,
                message: "fee rate not yet available",
            }),
        )
            .into_response(),
    }
}

async fn get_health() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
struct BroadcastRequest {
    tx_hex: String,
}

#[derive(Serialize)]
struct BroadcastResponse {
    txid: Option<String>,
    vsize: Option<u64>,
    status: &'static str,
    error: Option<String>,
}

#[derive(Serialize)]
struct RateLimitedResponse {
    error: &'static str,
    retry_after_secs: u64,
}

async fn post_broadcast(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(request): Json<BroadcastRequest>,
) -> impl IntoResponse {
    let ip = rate_limit::resolve_client_ip(&headers, peer);
    let tx_hex = request.tx_hex.trim();
    let tx = match tx_core::decode_as(tx_core::Format::TxHex, tx_hex.as_bytes()) {
        Ok(tx_core::Decoded::Transaction(tx)) if tx.input.is_empty() => {
            return broadcast_response(
                StatusCode::BAD_REQUEST,
                None,
                None,
                "invalid",
                Some("transaction has no inputs".to_string()),
            );
        }
        Ok(tx_core::Decoded::Transaction(tx)) if tx.output.is_empty() => {
            return broadcast_response(
                StatusCode::BAD_REQUEST,
                None,
                None,
                "invalid",
                Some("transaction has no outputs".to_string()),
            );
        }
        Ok(tx_core::Decoded::Transaction(tx)) => tx,
        Ok(_) => unreachable!("decode_as(Format::TxHex, _) always returns a raw transaction"),
        Err(err) => {
            return broadcast_response(
                StatusCode::BAD_REQUEST,
                None,
                None,
                "invalid",
                Some(err.to_string()),
            );
        }
    };

    let vsize = tx_core::vsize(&tx);
    let txid = tx.compute_txid().to_string();
    state.rate_limiter.record(ip);

    match state.slipstream.submit_tx(&request.tx_hex).await {
        Ok(_) => broadcast_response(StatusCode::OK, Some(txid), Some(vsize), "submitted", None),
        Err(SlipstreamError::Rejected(message)) => broadcast_response(
            StatusCode::OK,
            Some(txid),
            Some(vsize),
            "rejected",
            Some(message),
        ),
        Err(SlipstreamError::ClientCode(message)) => broadcast_response(
            StatusCode::BAD_GATEWAY,
            Some(txid),
            Some(vsize),
            "rejected",
            Some(message),
        ),
        Err(err @ (SlipstreamError::Transport(_) | SlipstreamError::Http { .. })) => {
            broadcast_response(
                StatusCode::BAD_GATEWAY,
                Some(txid),
                Some(vsize),
                "rejected",
                Some(err.to_string()),
            )
        }
    }
}

fn broadcast_response(
    status_code: StatusCode,
    txid: Option<String>,
    vsize: Option<u64>,
    status: &'static str,
    error: Option<String>,
) -> axum::response::Response {
    (
        status_code,
        Json(BroadcastResponse {
            txid,
            vsize,
            status,
            error,
        }),
    )
        .into_response()
}

fn rate_limited_response(retry_after: Duration) -> axum::response::Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(RateLimitedResponse {
            error: "rate limited",
            retry_after_secs: retry_after.as_secs(),
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use chrono::Duration as ChronoDuration;
    use serde_json::json;
    use slipstream_client::SlipstreamConfig;
    use tower::ServiceExt;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const TEST_MAX_PAYLOAD_BYTES: usize = 1_048_576;

    fn test_state(fee_cache: FeeCache) -> AppState {
        let slipstream_config = SlipstreamConfig {
            base_url: "http://127.0.0.1:1".to_string(),
            fee_endpoint: "/api/rates".to_string(),
            submit_endpoint: "/api/transactions".to_string(),
            client_code: String::new(),
            request_timeout_secs: 1,
        };
        AppState {
            slipstream: Arc::new(SlipstreamClient::new(&slipstream_config)),
            fee_cache,
            rate_limiter: RateLimiter::new(Duration::from_secs(600), 100),
        }
    }

    fn broadcast_state(base_url: String, rate_limiter: RateLimiter) -> AppState {
        let slipstream_config = SlipstreamConfig {
            base_url,
            fee_endpoint: "/api/rates".to_string(),
            submit_endpoint: "/api/transactions".to_string(),
            client_code: "TESTCODE".to_string(),
            request_timeout_secs: 1,
        };
        AppState {
            slipstream: Arc::new(SlipstreamClient::new(&slipstream_config)),
            fee_cache: FeeCache::new(),
            rate_limiter,
        }
    }

    fn sample_transaction(locktime: u32) -> bitcoin::Transaction {
        bitcoin::Transaction {
            version: bitcoin::transaction::Version::ONE,
            lock_time: bitcoin::absolute::LockTime::from_consensus(locktime),
            input: vec![bitcoin::TxIn {
                previous_output: bitcoin::OutPoint::null(),
                script_sig: bitcoin::ScriptBuf::new(),
                sequence: bitcoin::Sequence::MAX,
                witness: bitcoin::Witness::new(),
            }],
            output: vec![bitcoin::TxOut {
                value: bitcoin::Amount::from_sat(5_000_000_000),
                script_pubkey: bitcoin::ScriptBuf::new(),
            }],
        }
    }

    fn sample_tx_hex() -> String {
        bitcoin::consensus::encode::serialize_hex(&sample_transaction(0))
    }

    fn tx_hex_with_no_inputs() -> String {
        let mut tx = sample_transaction(0);
        tx.input.clear();
        bitcoin::consensus::encode::serialize_hex(&tx)
    }

    fn tx_hex_with_no_outputs() -> String {
        let mut tx = sample_transaction(0);
        tx.output.clear();
        bitcoin::consensus::encode::serialize_hex(&tx)
    }

    fn non_local_peer(last_octet: u8) -> SocketAddr {
        SocketAddr::from(([203, 0, 113, last_octet], 9000))
    }

    fn broadcast_request(body: serde_json::Value, peer: SocketAddr) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/broadcast")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .extension(ConnectInfo(peer))
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn fee_served_from_prepopulated_cache() {
        let fee_cache = FeeCache::new();
        fee_cache
            .record_success(FeeInfo {
                effective_rate_sat_vb: 4.0,
                fetched_at: Utc::now(),
            })
            .await;
        let app = router(test_state(fee_cache), TEST_MAX_PAYLOAD_BYTES);

        let response = app
            .oneshot(Request::builder().uri("/fee").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["effective_rate_sat_vb"], 4.0);
        assert_eq!(json["stale"], false);
        assert_eq!(json["age_secs"], 0);
    }

    #[tokio::test]
    async fn fee_reports_stale_after_three_consecutive_failures() {
        let fee_cache = FeeCache::new();
        fee_cache
            .record_success(FeeInfo {
                effective_rate_sat_vb: 4.0,
                fetched_at: Utc::now() - ChronoDuration::seconds(120),
            })
            .await;
        for _ in 0..2 {
            fee_cache.record_failure().await;
        }
        let app = router(test_state(fee_cache.clone()), TEST_MAX_PAYLOAD_BYTES);
        let response = app
            .oneshot(Request::builder().uri("/fee").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(body_json(response).await["stale"], false);

        fee_cache.record_failure().await;
        let app = router(test_state(fee_cache), TEST_MAX_PAYLOAD_BYTES);
        let response = app
            .oneshot(Request::builder().uri("/fee").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["stale"], true);
        assert_eq!(json["effective_rate_sat_vb"], 4.0);
    }

    #[tokio::test]
    async fn fee_before_first_successful_poll() {
        let app = router(test_state(FeeCache::new()), TEST_MAX_PAYLOAD_BYTES);
        let response = app
            .oneshot(Request::builder().uri("/fee").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let json = body_json(response).await;
        assert_eq!(json["stale"], true);
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let app = router(test_state(FeeCache::new()), TEST_MAX_PAYLOAD_BYTES);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn broadcast_valid_hex_submitted_returns_submitted_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/transactions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"status":"success","message":"accepted"}"#),
            )
            .mount(&server)
            .await;

        let state = broadcast_state(
            server.uri(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);
        let expected_txid = sample_transaction(0).compute_txid().to_string();

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": sample_tx_hex()}),
                non_local_peer(1),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["status"], "submitted");
        assert_eq!(json["txid"], expected_txid);
        assert!(json["vsize"].as_u64().unwrap() > 0);
        assert_eq!(json["error"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn broadcast_slipstream_rejection_surfaces_message_verbatim() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/transactions"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"{"status":"error","message":"fee too low: 1.0 sat/vB below floor 4.0 sat/vB"}"#,
            ))
            .mount(&server)
            .await;

        let state = broadcast_state(
            server.uri(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": sample_tx_hex()}),
                non_local_peer(2),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let json = body_json(response).await;
        assert_eq!(json["status"], "rejected");
        assert_eq!(
            json["error"],
            "fee too low: 1.0 sat/vB below floor 4.0 sat/vB"
        );
    }

    #[tokio::test]
    async fn broadcast_slipstream_unreachable_returns_502() {
        let state = broadcast_state(
            "http://127.0.0.1:1".to_string(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": sample_tx_hex()}),
                non_local_peer(3),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let json = body_json(response).await;
        assert_eq!(json["status"], "rejected");
        assert!(json["error"].as_str().unwrap().contains("transport error"));
    }

    #[tokio::test]
    async fn broadcast_malformed_hex_is_invalid_and_makes_no_outbound_call() {
        let server = MockServer::start().await;
        let state = broadcast_state(
            server.uri(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": "not-hex-at-all"}),
                non_local_peer(4),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let json = body_json(response).await;
        assert_eq!(json["status"], "invalid");
        assert_eq!(json["txid"], serde_json::Value::Null);
        assert_eq!(json["vsize"], serde_json::Value::Null);
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn broadcast_zero_inputs_is_invalid() {
        let state = broadcast_state(
            "http://127.0.0.1:1".to_string(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": tx_hex_with_no_inputs()}),
                non_local_peer(5),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(body_json(response).await["status"], "invalid");
    }

    #[tokio::test]
    async fn broadcast_zero_outputs_is_invalid() {
        let state = broadcast_state(
            "http://127.0.0.1:1".to_string(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": tx_hex_with_no_outputs()}),
                non_local_peer(6),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(body_json(response).await["status"], "invalid");
    }

    #[tokio::test]
    async fn broadcast_oversize_body_returns_413() {
        let state = broadcast_state(
            "http://127.0.0.1:1".to_string(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, 16);

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": sample_tx_hex()}),
                non_local_peer(7),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn broadcast_body_limit_allows_configured_hex_length() {
        let state = broadcast_state(
            "http://127.0.0.1:1".to_string(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, 16);

        let response = app
            .oneshot(broadcast_request(
                json!({"tx_hex": "00".repeat(8)}),
                non_local_peer(7),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn broadcast_rate_limit_exhaustion_returns_429_then_recovers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/transactions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"status":"success","message":"accepted"}"#),
            )
            .mount(&server)
            .await;

        let window = Duration::from_millis(300);
        let state = broadcast_state(server.uri(), RateLimiter::new(window, 1));
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);
        let peer = non_local_peer(8);

        let first = app
            .clone()
            .oneshot(broadcast_request(json!({"tx_hex": sample_tx_hex()}), peer))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::OK);

        let second = app
            .clone()
            .oneshot(broadcast_request(json!({"tx_hex": sample_tx_hex()}), peer))
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
        let json = body_json(second).await;
        assert!(json["retry_after_secs"].as_u64().unwrap() <= window.as_secs() + 1);

        tokio::time::sleep(window + Duration::from_millis(50)).await;

        let third = app
            .oneshot(broadcast_request(json!({"tx_hex": sample_tx_hex()}), peer))
            .await
            .unwrap();
        assert_eq!(third.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn broadcast_invalid_hex_does_not_consume_rate_limit_allowance() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/transactions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"status":"success","message":"accepted"}"#),
            )
            .mount(&server)
            .await;

        let state = broadcast_state(server.uri(), RateLimiter::new(Duration::from_secs(600), 1));
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);
        let peer = non_local_peer(9);

        let invalid = app
            .clone()
            .oneshot(broadcast_request(json!({"tx_hex": "not-hex"}), peer))
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

        let valid = app
            .oneshot(broadcast_request(json!({"tx_hex": sample_tx_hex()}), peer))
            .await
            .unwrap();
        assert_eq!(valid.status(), StatusCode::OK);
        assert_eq!(body_json(valid).await["status"], "submitted");
    }

    #[tokio::test]
    async fn broadcast_two_dependent_transactions_arrive_at_mock_in_order() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/transactions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"status":"success","message":"accepted"}"#),
            )
            .mount(&server)
            .await;

        let state = broadcast_state(
            server.uri(),
            RateLimiter::new(Duration::from_secs(600), 100),
        );
        let app = router(state, TEST_MAX_PAYLOAD_BYTES);
        let peer = non_local_peer(10);

        let parent_hex = bitcoin::consensus::encode::serialize_hex(&sample_transaction(1));
        let child_hex = bitcoin::consensus::encode::serialize_hex(&sample_transaction(2));

        let parent_response = app
            .clone()
            .oneshot(broadcast_request(json!({"tx_hex": parent_hex}), peer))
            .await
            .unwrap();
        assert_eq!(parent_response.status(), StatusCode::OK);

        let child_response = app
            .oneshot(broadcast_request(json!({"tx_hex": child_hex}), peer))
            .await
            .unwrap();
        assert_eq!(child_response.status(), StatusCode::OK);

        let received = server.received_requests().await.unwrap();
        assert_eq!(received.len(), 2);
        let bodies: Vec<serde_json::Value> = received
            .iter()
            .map(|request| request.body_json().unwrap())
            .collect();
        assert_eq!(bodies[0]["tx_hex"], parent_hex);
        assert_eq!(bodies[1]["tx_hex"], child_hex);
    }
}
