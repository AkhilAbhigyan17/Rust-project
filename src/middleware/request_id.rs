use axum::body::Body;
use axum::http::{HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use uuid::Uuid;

pub async fn add_request_id(mut req: Request<Body>, next: Next) -> Response {
    let rid = Uuid::new_v4().to_string();
    if let Ok(v) = HeaderValue::from_str(&rid) {
        req.headers_mut().insert("x-request-id", v.clone());
        let mut res = next.run(req).await;
        res.headers_mut().insert("x-request-id", v);
        return res;
    }
    next.run(req).await
}
