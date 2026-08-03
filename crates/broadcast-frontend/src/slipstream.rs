//! Browser-side access to MARA's Slipstream API. Slipstream serves
//! permissive CORS on both endpoints used here, so the frontend talks to it
//! directly instead of routing through the backend.

use gloo_net::http::{Request, Response};
use serde::{Deserialize, Serialize};

/// Overridable at build time so a deployment can point at another host.
const BASE_URL: &str = match option_env!("SLIPSTREAM_BASE_URL") {
    Some(url) => url,
    None => "https://slipstream.mara.com",
};
const RATES_PATH: &str = "/api/rates";
const SUBMIT_PATH: &str = "/api/transactions";

const TOO_MANY_REQUESTS: u16 = 429;
/// How many times a rate-limited transaction is re-sent before giving up.
const MAX_RATE_LIMIT_RETRIES: u32 = 3;
const FIRST_RETRY_DELAY_SECS: u64 = 5;
const MIN_RETRY_DELAY_SECS: u64 = 1;
const MAX_RETRY_DELAY_SECS: u64 = 300;

/// A rejected fetch is opaque by design: the browser reports CORS, DNS, TLS,
/// an offline tab and a blocked request identically, so the copy names what
/// usually causes one instead of repeating a message that says nothing.
const UNREACHABLE: &str = "Could not reach MARA Slipstream. Check your connection, or whether a VPN, proxy, or browser extension is blocking slipstream.mara.com.";

#[derive(Deserialize)]
struct RatesResponse {
    /// The rate at which a submission actually gets mined, as opposed to
    /// `submit_fee_rate`, which only buys admission to the private mempool.
    effective_rate: f64,
}

#[derive(Serialize)]
struct SubmitRequest<'a> {
    tx_hex: &'a str,
}

/// The `{status, message}` shape `/api/transactions` returns on both 200 and
/// 400.
#[derive(Deserialize)]
struct SubmitResponse {
    status: String,
    message: String,
}

/// What one submission attempt resolved to.
#[derive(Debug, PartialEq, Eq)]
pub enum SubmitOutcome {
    Accepted,
    /// Slipstream answered with a reason of its own, so sending the same
    /// transaction again cannot change the answer.
    Rejected(String),
    /// The suggested delay comes from `Retry-After`, which is not
    /// CORS-safelisted and so is usually unreadable from a browser.
    RateLimited(Option<u64>),
    /// No usable answer came back: the request never arrived, or Slipstream
    /// failed transiently on its side.
    Failed(String),
}

/// `GET /api/rates`. The client code is omitted: it only applies volume
/// discounts, and the frontend holds none.
pub async fn fetch_rates() -> Option<f64> {
    let url = format!("{BASE_URL}{RATES_PATH}");
    let response = Request::get(&url).send().await.ok()?;
    if !response.ok() {
        return None;
    }
    let body: RatesResponse = response.json().await.ok()?;
    Some(body.effective_rate)
}

/// `POST /api/transactions` with `{tx_hex}`. No client code is sent: it is
/// optional upstream, and a browser has no secret to hold one in.
pub async fn submit_tx(tx_hex: &str) -> SubmitOutcome {
    let url = format!("{BASE_URL}{SUBMIT_PATH}");
    let request = match Request::post(&url).json(&SubmitRequest { tx_hex }) {
        Ok(request) => request,
        Err(err) => return SubmitOutcome::Failed(err.to_string()),
    };
    let Ok(response) = request.send().await else {
        return SubmitOutcome::Failed(UNREACHABLE.to_string());
    };
    // An unreadable body lands on the unexpected-response arm of classify().
    let body = response.text().await.unwrap_or_default();

    match classify(response.status(), &body) {
        // classify() sees no headers, so the suggested delay is filled in here.
        SubmitOutcome::RateLimited(_) => SubmitOutcome::RateLimited(retry_after_secs(&response)),
        outcome => outcome,
    }
}

/// Slipstream's answer as a single outcome, pure so the whole ladder is
/// testable without HTTP.
fn classify(status: u16, body: &str) -> SubmitOutcome {
    if status == TOO_MANY_REQUESTS {
        return SubmitOutcome::RateLimited(None);
    }
    let Ok(parsed) = serde_json::from_str::<SubmitResponse>(body) else {
        return unexpected(status);
    };

    match (status, parsed.status.as_str()) {
        (200..=299, "success") => SubmitOutcome::Accepted,
        (200..=299, "error") => SubmitOutcome::Rejected(parsed.message),
        (400..=499, _) => SubmitOutcome::Rejected(parsed.message),
        (500..=599, _) => SubmitOutcome::Failed(parsed.message),
        _ => unexpected(status),
    }
}

fn unexpected(status: u16) -> SubmitOutcome {
    SubmitOutcome::Failed(format!(
        "Unexpected response from Slipstream (HTTP {status})"
    ))
}

/// `Retry-After` in seconds, when the browser exposes it at all. The HTTP-date
/// form is not read: Slipstream sends the delta-seconds form.
fn retry_after_secs(response: &Response) -> Option<u64> {
    response.headers().get("retry-after")?.trim().parse().ok()
}

/// How long to wait before rate-limit retry number `attempt`, counting from
/// zero, or None once the retries are spent. A suggested delay is clamped
/// because it comes from a host nobody here controls: an hour-long one would
/// read as a frozen queue.
pub fn retry_delay(attempt: u32, suggested: Option<u64>) -> Option<u64> {
    if attempt >= MAX_RATE_LIMIT_RETRIES {
        return None;
    }
    let delay = suggested.unwrap_or(FIRST_RETRY_DELAY_SECS * 2u64.pow(attempt));
    Some(delay.clamp(MIN_RETRY_DELAY_SECS, MAX_RETRY_DELAY_SECS))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `/api/rates` body recorded live on 2026-08-02.
    const RATES_BODY: &str = r#"{"market_rate":2.0, "multiplier":2.0, "multiplier_discount_percent":0,
 "discounted_multiplier":2.0, "submit_fee_rate":2.0,
 "slipstream_rate":4.0, "effective_rate":4.0}"#;

    /// An `/api/transactions` 400 body recorded live on 2026-08-02, back when
    /// a client code was still required to submit.
    const REJECTION_BODY: &str = r#"{"status":"error","message":"Client codes are currently required to submit transactions. Contact us at foundation@mara.com for more information."}"#;
    const REJECTION_MESSAGE: &str = "Client codes are currently required to submit transactions. Contact us at foundation@mara.com for more information.";

    #[test]
    fn recorded_rates_body_parses() {
        let parsed: RatesResponse = serde_json::from_str(RATES_BODY).expect("rates body parses");
        assert_eq!(parsed.effective_rate, 4.0);
    }

    #[test]
    fn success_on_200_is_accepted() {
        assert_eq!(
            classify(
                200,
                r#"{"status":"success","message":"Transaction accepted"}"#
            ),
            SubmitOutcome::Accepted
        );
    }

    #[test]
    fn error_on_200_is_rejected() {
        assert_eq!(
            classify(200, r#"{"status":"error","message":"fee rate too low"}"#),
            SubmitOutcome::Rejected("fee rate too low".to_string())
        );
    }

    #[test]
    fn recorded_400_body_is_rejected_with_its_message() {
        assert_eq!(
            classify(400, REJECTION_BODY),
            SubmitOutcome::Rejected(REJECTION_MESSAGE.to_string())
        );
    }

    #[test]
    fn too_many_requests_is_rate_limited_whatever_the_body() {
        assert_eq!(classify(429, ""), SubmitOutcome::RateLimited(None));
        assert_eq!(
            classify(429, REJECTION_BODY),
            SubmitOutcome::RateLimited(None)
        );
    }

    #[test]
    fn server_error_carries_its_message() {
        assert_eq!(
            classify(
                503,
                r#"{"status":"error","message":"upstream unavailable"}"#
            ),
            SubmitOutcome::Failed("upstream unavailable".to_string())
        );
    }

    #[test]
    fn unparseable_body_names_the_status() {
        assert_eq!(
            classify(502, "<html>bad gateway</html>"),
            SubmitOutcome::Failed("Unexpected response from Slipstream (HTTP 502)".to_string())
        );
        assert_eq!(
            classify(200, ""),
            SubmitOutcome::Failed("Unexpected response from Slipstream (HTTP 200)".to_string())
        );
    }

    #[test]
    fn unknown_status_string_on_200_is_unexpected() {
        assert_eq!(
            classify(200, r#"{"status":"queued","message":"waiting"}"#),
            SubmitOutcome::Failed("Unexpected response from Slipstream (HTTP 200)".to_string())
        );
    }

    #[test]
    fn retry_delay_backs_off_then_gives_up() {
        assert_eq!(retry_delay(0, None), Some(5));
        assert_eq!(retry_delay(1, None), Some(10));
        assert_eq!(retry_delay(2, None), Some(20));
        assert_eq!(retry_delay(3, None), None);
        assert_eq!(retry_delay(3, Some(5)), None);
    }

    #[test]
    fn suggested_delay_is_used_within_its_bounds() {
        assert_eq!(retry_delay(0, Some(45)), Some(45));
    }

    #[test]
    fn suggested_delay_is_clamped() {
        assert_eq!(retry_delay(0, Some(0)), Some(1));
        assert_eq!(retry_delay(0, Some(86_400)), Some(300));
    }
}
