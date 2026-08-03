//! Per-IP sliding-window rate limiter for `POST /broadcast` (PLAN.md
//! section 4, "Rate limiting"). Each IP may attempt at most `max_tx`
//! submissions within any rolling `window`; the map self-cleans periodically
//! on access, so no background sweep task is needed.

use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::http::HeaderMap;

const LOCALHOST_V4: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const LOCALHOST_V6: IpAddr = IpAddr::V6(Ipv6Addr::LOCALHOST);

fn is_exempt(ip: IpAddr) -> bool {
    ip == LOCALHOST_V4 || ip == LOCALHOST_V6
}

#[derive(Clone)]
pub struct RateLimiter {
    window: Duration,
    max_tx: usize,
    state: std::sync::Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    requests: HashMap<IpAddr, VecDeque<Instant>>,
    last_cleanup: Instant,
}

impl RateLimiter {
    pub fn new(window: Duration, max_tx: usize) -> Self {
        let now = Instant::now();
        Self {
            window,
            max_tx,
            state: std::sync::Arc::new(Mutex::new(RateLimitState {
                requests: HashMap::new(),
                last_cleanup: now,
            })),
        }
    }

    /// Prunes the current IP, then atomically admits and records its request.
    /// Stale IPs are removed periodically. Localhost is always exempt.
    pub fn check_and_record(&self, ip: IpAddr) -> Result<(), Duration> {
        if is_exempt(ip) {
            return Ok(());
        }

        let mut state = self.state.lock().unwrap();
        let now = Instant::now();
        prune_ip(&mut state.requests, ip, self.window, now);
        if now.duration_since(state.last_cleanup) >= self.window {
            prune_all(&mut state.requests, self.window, now);
            state.last_cleanup = now;
        }
        let deque = state.requests.entry(ip).or_default();

        if deque.len() < self.max_tx {
            deque.push_back(now);
            Ok(())
        } else {
            let oldest = *deque.front().expect("len >= max_tx > 0");
            Err(self.window.saturating_sub(now.duration_since(oldest)))
        }
    }
}

fn prune_ip(
    requests: &mut HashMap<IpAddr, VecDeque<Instant>>,
    ip: IpAddr,
    window: Duration,
    now: Instant,
) {
    if let Some(deque) = requests.get_mut(&ip) {
        prune_deque(deque, window, now);
        if deque.is_empty() {
            requests.remove(&ip);
        }
    }
}

fn prune_all(requests: &mut HashMap<IpAddr, VecDeque<Instant>>, window: Duration, now: Instant) {
    requests.retain(|_, deque| {
        prune_deque(deque, window, now);
        !deque.is_empty()
    });
}

fn prune_deque(deque: &mut VecDeque<Instant>, window: Duration, now: Instant) {
    while let Some(&oldest) = deque.front() {
        if now.duration_since(oldest) >= window {
            deque.pop_front();
        } else {
            break;
        }
    }
}

/// Trusts proxy headers only from a local nginx peer. nginx overwrites
/// `X-Real-IP`; the last `X-Forwarded-For` entry is its fallback address.
pub fn resolve_client_ip(headers: &HeaderMap, peer: SocketAddr) -> IpAddr {
    if !is_exempt(peer.ip()) {
        return peer.ip();
    }

    if let Some(ip) = headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<IpAddr>().ok())
    {
        return ip;
    }

    if let Some(ip) = headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.rsplit(',').next())
        .and_then(|last| last.trim().parse::<IpAddr>().ok())
    {
        return ip;
    }

    peer.ip()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    fn peer(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    #[test]
    fn exhaust_then_block() {
        let limiter = RateLimiter::new(Duration::from_secs(600), 3);
        let client = ip("10.0.0.1");
        for _ in 0..3 {
            assert!(limiter.check_and_record(client).is_ok());
        }
        assert!(limiter.check_and_record(client).is_err());
    }

    #[test]
    fn blocked_call_reports_sane_remaining_duration() {
        let window = Duration::from_secs(600);
        let limiter = RateLimiter::new(window, 1);
        let client = ip("10.0.0.2");
        limiter.check_and_record(client).unwrap();
        let remaining = limiter.check_and_record(client).unwrap_err();
        assert!(remaining > Duration::ZERO);
        assert!(remaining <= window);
    }

    #[tokio::test]
    async fn recovers_after_window_elapses() {
        let window = Duration::from_millis(50);
        let limiter = RateLimiter::new(window, 1);
        let client = ip("10.0.0.3");
        assert!(limiter.check_and_record(client).is_ok());
        assert!(limiter.check_and_record(client).is_err());

        tokio::time::sleep(window + Duration::from_millis(10)).await;
        assert!(limiter.check_and_record(client).is_ok());
    }

    #[test]
    fn distinct_ips_are_independent() {
        let limiter = RateLimiter::new(Duration::from_secs(600), 1);
        let a = ip("10.0.0.4");
        let b = ip("10.0.0.5");
        limiter.record(a);
        assert!(limiter.check(a).is_err());
        assert!(limiter.check(b).is_ok());
    }

    #[test]
    fn localhost_is_exempt_beyond_max_tx() {
        let limiter = RateLimiter::new(Duration::from_secs(600), 1);
        for _ in 0..10 {
            assert!(limiter.check(LOCALHOST_V4).is_ok());
            limiter.record(LOCALHOST_V4);
            assert!(limiter.check(LOCALHOST_V6).is_ok());
            limiter.record(LOCALHOST_V6);
        }
    }

    #[test]
    fn check_without_record_does_not_consume_allowance() {
        let limiter = RateLimiter::new(Duration::from_secs(600), 1);
        let client = ip("10.0.0.6");
        assert!(limiter.check(client).is_ok());
        assert!(limiter.check(client).is_ok());
        limiter.record(client);
        assert!(limiter.check(client).is_err());
    }

    #[tokio::test]
    async fn map_does_not_grow_unboundedly() {
        let window = Duration::from_millis(50);
        let limiter = RateLimiter::new(window, 5);
        let client = ip("10.0.0.7");
        limiter.record(client);
        assert_eq!(limiter.requests.lock().unwrap().len(), 1);

        tokio::time::sleep(window + Duration::from_millis(10)).await;
        assert!(limiter.check(client).is_ok());
        assert_eq!(limiter.requests.lock().unwrap().len(), 0);
    }

    #[test]
    fn resolve_ip_prefers_first_x_forwarded_for_entry() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.5, 10.0.0.1"),
        );
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.9"));
        assert_eq!(
            resolve_client_ip(&headers, peer("127.0.0.1:1234")),
            ip("203.0.113.5")
        );
    }

    #[test]
    fn resolve_ip_falls_back_to_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.9"));
        assert_eq!(
            resolve_client_ip(&headers, peer("127.0.0.1:1234")),
            ip("198.51.100.9")
        );
    }

    #[test]
    fn resolve_ip_falls_back_to_peer_addr() {
        let headers = HeaderMap::new();
        assert_eq!(
            resolve_client_ip(&headers, peer("192.0.2.1:1234")),
            ip("192.0.2.1")
        );
    }

    #[test]
    fn resolve_ip_falls_through_malformed_x_forwarded_for() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("not-an-ip"));
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.9"));
        assert_eq!(
            resolve_client_ip(&headers, peer("127.0.0.1:1234")),
            ip("198.51.100.9")
        );
    }
}
