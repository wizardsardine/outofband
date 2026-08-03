//! Browser-side access to MARA's Slipstream API. Slipstream serves
//! permissive CORS on both endpoints used here, so the frontend talks to it
//! directly instead of routing through the backend.

use gloo_net::http::Request;
use serde::Deserialize;

/// Overridable at build time so a deployment can point at another host.
const BASE_URL: &str = match option_env!("SLIPSTREAM_BASE_URL") {
    Some(url) => url,
    None => "https://slipstream.mara.com",
};
const RATES_PATH: &str = "/api/rates";
const SUBMIT_PATH: &str = "/api/transactions";

#[derive(Deserialize)]
struct RatesResponse {
    /// The rate at which a submission actually gets mined, as opposed to
    /// `submit_fee_rate`, which only buys admission to the private mempool.
    effective_rate: f64,
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
