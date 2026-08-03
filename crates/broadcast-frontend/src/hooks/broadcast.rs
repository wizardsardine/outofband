//! The single `POST /broadcast` call, isolated from the loop that drives it
//! ([`super::queue`]) so the HTTP/JSON shape lives in one place.

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

/// What one `/broadcast` attempt resolved to. `Rejected` covers every
/// definitive answer from the server — including a Slipstream-side outage
/// (502) or a locally-impossible `invalid`, both of which still arrive as a
/// well-formed body — so only a request that never got a body of its own
/// counts as `Failed`.
pub enum SubmitOutcome {
    Accepted,
    Rejected(String),
    RateLimited(u64),
    Failed(String),
}

#[derive(Serialize)]
struct BroadcastRequest<'a> {
    tx_hex: &'a str,
}

#[derive(Deserialize)]
struct BroadcastResponseBody {
    status: String,
    error: Option<String>,
}

#[derive(Deserialize)]
struct RateLimitedBody {
    retry_after_secs: u64,
}

pub async fn submit_tx(tx_hex: &str) -> SubmitOutcome {
    let request = match Request::post("/broadcast").json(&BroadcastRequest { tx_hex }) {
        Ok(request) => request,
        Err(err) => return SubmitOutcome::Failed(err.to_string()),
    };
    let response = match request.send().await {
        Ok(response) => response,
        Err(err) => return SubmitOutcome::Failed(err.to_string()),
    };

    if response.status() == 429 {
        return match response.json::<RateLimitedBody>().await {
            Ok(body) => SubmitOutcome::RateLimited(body.retry_after_secs),
            Err(err) => SubmitOutcome::Failed(err.to_string()),
        };
    }

    match response.json::<BroadcastResponseBody>().await {
        Ok(body) if body.status == "submitted" => SubmitOutcome::Accepted,
        Ok(body) => SubmitOutcome::Rejected(
            body.error
                .unwrap_or_else(|| "submission rejected".to_string()),
        ),
        Err(err) => SubmitOutcome::Failed(err.to_string()),
    }
}
