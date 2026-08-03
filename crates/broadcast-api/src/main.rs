mod config;
mod rate_limit;
mod routes;

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use slipstream_client::SlipstreamClient;
use tokio::time::interval;

use crate::config::{config_path, load_config};
use crate::rate_limit::RateLimiter;
use crate::routes::{AppState, FeeCache, router};

#[tokio::main]
async fn main() -> ExitCode {
    let path = config_path();
    let config = match load_config(&path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("broadcast-api: {err}");
            return ExitCode::FAILURE;
        }
    };

    let slipstream = Arc::new(SlipstreamClient::new(&config.slipstream));
    let fee_cache = FeeCache::new();
    spawn_fee_poller(
        slipstream.clone(),
        fee_cache.clone(),
        config.broadcast_api.fee_poll_secs,
    );
    let rate_limiter = RateLimiter::new(
        config.broadcast_api.rate_limit_window_secs,
        config.broadcast_api.rate_limit_max_tx,
    );

    let app = router(AppState {
        slipstream,
        fee_cache,
        rate_limiter,
    });

    let listener = match tokio::net::TcpListener::bind(config.broadcast_api.listen_addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!(
                "broadcast-api: could not bind {}: {err}",
                config.broadcast_api.listen_addr
            );
            return ExitCode::FAILURE;
        }
    };

    println!(
        "broadcast-api: listening on {}",
        config.broadcast_api.listen_addr
    );
    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("broadcast-api: server error: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

/// Refreshes the fee cache every `poll_interval` by calling
/// `SlipstreamClient::rates()`. Handlers never call Slipstream themselves —
/// this task is the cache's sole writer.
fn spawn_fee_poller(client: Arc<SlipstreamClient>, cache: FeeCache, poll_interval: Duration) {
    tokio::spawn(async move {
        let mut ticker = interval(poll_interval);
        loop {
            ticker.tick().await;
            match client.rates().await {
                Ok(fee) => cache.record_success(fee).await,
                Err(_) => cache.record_failure().await,
            }
        }
    });
}
