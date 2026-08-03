use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use serde::Deserialize;
use yew::prelude::*;

use crate::tokens::FEE_POLL_INTERVAL_MS;

#[derive(Clone, PartialEq)]
pub struct FeeSnapshot {
    pub rate_sat_vb: f64,
    pub stale: bool,
}

#[derive(Deserialize)]
struct FeeResponse {
    effective_rate_sat_vb: f64,
    stale: bool,
}

/// Polls `GET /fee` every [`FEE_POLL_INTERVAL_MS`]. A failed poll never
/// wipes the last known rate: it is kept and marked stale, so the fee card
/// can dim a number instead of presenting a dead one as live.
#[hook]
pub fn use_fee() -> Option<FeeSnapshot> {
    let fee = use_state(|| None::<FeeSnapshot>);

    {
        let fee = fee.clone();
        use_effect_with((), move |()| {
            wasm_bindgen_futures::spawn_local(async move {
                loop {
                    match fetch_fee().await {
                        Some(snapshot) => fee.set(Some(snapshot)),
                        None => {
                            let stale_previous = (*fee).clone().map(|previous| FeeSnapshot {
                                stale: true,
                                ..previous
                            });
                            fee.set(stale_previous);
                        }
                    }
                    TimeoutFuture::new(FEE_POLL_INTERVAL_MS).await;
                }
            });
            || ()
        });
    }

    (*fee).clone()
}

async fn fetch_fee() -> Option<FeeSnapshot> {
    let response = Request::get("/fee").send().await.ok()?;
    if !response.ok() {
        return None;
    }
    let body: FeeResponse = response.json().await.ok()?;
    Some(FeeSnapshot {
        rate_sat_vb: body.effective_rate_sat_vb,
        stale: body.stale,
    })
}
