//! `AppState`, the fee cache, and the routes that need no transaction
//! handling: `GET /fee` and `GET /health`. `POST /broadcast` joins this
//! module in a later phase; the rate limiter it will use is already part
//! of `AppState`.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::Serialize;
use slipstream_client::{FeeInfo, SlipstreamClient};
use tokio::sync::RwLock;

use crate::rate_limit::RateLimiter;

/// After this many consecutive failed refresh attempts, `/fee` reports
/// `stale: true` alongside the last-known value rather than silently
/// serving a number that may no longer reflect Slipstream's floor.
const STALE_AFTER_CONSECUTIVE_FAILURES: u32 = 3;

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

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/fee", get(get_fee))
        .route("/health", get(get_health))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use chrono::Duration as ChronoDuration;
    use slipstream_client::SlipstreamConfig;
    use tower::ServiceExt;

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
            rate_limiter: RateLimiter::new(std::time::Duration::from_secs(600), 100),
        }
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
        let app = router(test_state(fee_cache));

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
        let app = router(test_state(fee_cache.clone()));
        let response = app
            .oneshot(Request::builder().uri("/fee").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(body_json(response).await["stale"], false);

        fee_cache.record_failure().await;
        let app = router(test_state(fee_cache));
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
        let app = router(test_state(FeeCache::new()));
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
        let app = router(test_state(FeeCache::new()));
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
}
