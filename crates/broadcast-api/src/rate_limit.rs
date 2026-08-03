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
    use std::sync::{Arc, Barrier};

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
        limiter.check_and_record(a).unwrap();
        assert!(limiter.check_and_record(a).is_err());
        assert!(limiter.check_and_record(b).is_ok());
    }

    #[test]
    fn localhost_is_exempt_beyond_max_tx() {
        let limiter = RateLimiter::new(Duration::from_secs(600), 1);
        for _ in 0..10 {
            assert!(limiter.check_and_record(LOCALHOST_V4).is_ok());
            assert!(limiter.check_and_record(LOCALHOST_V6).is_ok());
        }
    }

    #[test]
    fn concurrent_requests_cannot_exceed_limit() {
        let max_tx = 4;
        let threads = 32;
        let limiter = RateLimiter::new(Duration::from_secs(600), max_tx);
        let client = ip("10.0.0.6");
        let barrier = Arc::new(Barrier::new(threads));
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let limiter = limiter.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    limiter.check_and_record(client).is_ok()
                })
            })
            .collect();

        let admitted = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|admitted| *admitted)
            .count();
        assert_eq!(admitted, max_tx);
    }

    #[tokio::test]
    async fn global_cleanup_is_periodic() {
        let window = Duration::from_millis(50);
        let limiter = RateLimiter::new(window, 1);
        for last_octet in 1..=200 {
            let client = IpAddr::V4(Ipv4Addr::new(10, 0, 0, last_octet));
            limiter.check_and_record(client).unwrap();
        }
        assert_eq!(limiter.state.lock().unwrap().requests.len(), 200);

        tokio::time::sleep(window + Duration::from_millis(10)).await;
        limiter.check_and_record(ip("10.0.1.1")).unwrap();
        assert_eq!(limiter.state.lock().unwrap().requests.len(), 1);
    }

    #[tokio::test]
    async fn current_ip_is_pruned_before_periodic_cleanup() {
        let window = Duration::from_millis(50);
        let limiter = RateLimiter::new(window, 1);
        let client = ip("10.0.0.1");
        limiter.check_and_record(client).unwrap();

        tokio::time::sleep(window + Duration::from_millis(10)).await;
        limiter.state.lock().unwrap().last_cleanup = Instant::now();

        assert!(limiter.check_and_record(client).is_ok());
        assert_eq!(limiter.state.lock().unwrap().requests.len(), 1);
    }

    #[test]
    fn resolve_ip_prefers_nginx_overwritten_x_real_ip() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("192.0.2.44, 203.0.113.5"),
        );
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.9"));
        assert_eq!(
            resolve_client_ip(&headers, peer("127.0.0.1:1234")),
            ip("198.51.100.9")
        );
    }

    #[test]
    fn resolve_ip_falls_back_to_last_x_forwarded_for_entry() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("not-an-ip"));
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("192.0.2.44, 198.51.100.9"),
        );
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
    fn resolve_ip_ignores_spoofed_headers_from_non_proxy_peer() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("192.0.2.44"));
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.9"));
        assert_eq!(
            resolve_client_ip(&headers, peer("203.0.113.5:1234")),
            ip("203.0.113.5")
        );
    }
}
