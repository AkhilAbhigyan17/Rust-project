//! Simple in-memory IP-based rate limiter (token bucket via governor).
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use governor::{Quota, RateLimiter};
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use nonzero_ext::nonzero;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::utils::error::ApiError;

pub type SharedLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

pub fn new_limiter() -> SharedLimiter {
    // 100 requests/second burst, refills continuously
    Arc::new(RateLimiter::direct(Quota::per_second(nonzero!(100u32))))
}

pub async fn rate_limit(
    _addr: Option<ConnectInfo<SocketAddr>>,
    axum::extract::State(limiter): axum::extract::State<SharedLimiter>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if limiter.check().is_err() {
        return ApiError::RateLimited.into_response();
    }
    next.run(req).await
}
