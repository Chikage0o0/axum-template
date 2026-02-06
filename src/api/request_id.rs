use axum::{
    extract::Request,
    http::{header::HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

tokio::task_local! {
    static REQUEST_ID: String;
}

pub const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(|id| id.clone()).ok()
}

fn request_id_from_header(req: &Request) -> Option<String> {
    req.headers()
        .get(&REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

pub async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let request_id = request_id_from_header(&req).unwrap_or_else(|| Uuid::new_v4().to_string());

    if req.headers().get(&REQUEST_ID_HEADER).is_none() {
        if let Ok(v) = HeaderValue::from_str(&request_id) {
            req.headers_mut().insert(REQUEST_ID_HEADER.clone(), v);
        }
    }

    let mut res = REQUEST_ID.scope(request_id.clone(), next.run(req)).await;

    if let Ok(v) = HeaderValue::from_str(&request_id) {
        res.headers_mut().insert(REQUEST_ID_HEADER.clone(), v);
    }

    res
}
