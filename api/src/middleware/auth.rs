use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use service::cookie::authorize;

pub async fn auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match get_cookie(&headers) {
        Some(cookie) if authorize(&cookie) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub fn get_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get("cookie")?;
    let cookie = cookie.to_str().ok()?;

    Some(cookie.to_string())
}
