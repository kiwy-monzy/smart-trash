//! Simple per-IP rate limiter for `/api/*` (fixed one-minute window).
//!
//! `RATE_LIMIT_PER_MINUTE=0` turns it off.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

const MAX_TRACKED_CLIENTS: usize = 10_000;

struct Bucket {
    window_start: Instant,
    count: u32,
}

#[derive(Clone)]
pub struct RateLimiter {
    limit: u32,
    window: Duration,
    buckets: Arc<Mutex<HashMap<IpAddr, Bucket>>>,
}

impl RateLimiter {
    pub fn new(limit_per_minute: u32) -> Self {
        Self {
            limit: limit_per_minute,
            window: Duration::from_secs(60),
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.limit > 0
    }

    /// Count one request; `Err(seconds)` means the window is used up.
    pub fn check(&self, client: IpAddr) -> Result<(), u64> {
        if !self.is_enabled() {
            return Ok(());
        }

        let now = Instant::now();
        let mut buckets = self
            .buckets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if buckets.len() > MAX_TRACKED_CLIENTS {
            let window = self.window;
            buckets.retain(|_, bucket| now.duration_since(bucket.window_start) < window);
        }

        let bucket = buckets.entry(client).or_insert_with(|| Bucket {
            window_start: now,
            count: 0,
        });

        if now.duration_since(bucket.window_start) >= self.window {
            bucket.window_start = now;
            bucket.count = 0;
        }

        if bucket.count >= self.limit {
            let elapsed = now.duration_since(bucket.window_start);
            return Err(self.window.saturating_sub(elapsed).as_secs().max(1));
        }

        bucket.count += 1;
        Ok(())
    }
}

fn client_ip(request: &Request) -> IpAddr {
    request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
}

pub async fn limit(State(limiter): State<RateLimiter>, request: Request, next: Next) -> Response {
    if !request.uri().path().starts_with("/api/") {
        return next.run(request).await;
    }

    match limiter.check(client_ip(&request)) {
        Ok(()) => next.run(request).await,
        Err(retry_after) => {
            let mut response =
                (StatusCode::TOO_MANY_REQUESTS, "too many requests, slow down").into_response();
            if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(last: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, last))
    }

    #[test]
    fn a_client_may_use_its_whole_window() {
        let limiter = RateLimiter::new(3);
        for _ in 0..3 {
            assert!(limiter.check(ip(1)).is_ok());
        }
        assert!(limiter.check(ip(1)).is_err());
    }

    #[test]
    fn clients_do_not_share_buckets() {
        let limiter = RateLimiter::new(1);
        assert!(limiter.check(ip(1)).is_ok());
        assert!(limiter.check(ip(2)).is_ok());
        assert!(limiter.check(ip(1)).is_err());
    }

    #[test]
    fn zero_disables_the_limiter() {
        let limiter = RateLimiter::new(0);
        for _ in 0..10_000 {
            assert!(limiter.check(ip(1)).is_ok());
        }
    }

    #[test]
    fn the_retry_hint_is_a_positive_number_of_seconds() {
        let limiter = RateLimiter::new(1);
        assert!(limiter.check(ip(1)).is_ok());
        let retry_after = limiter.check(ip(1)).expect_err("limited");
        assert!((1..=60).contains(&retry_after), "got {retry_after}");
    }
}
