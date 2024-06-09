use axum::http::HeaderMap;
use service::cookie::extract_cookie_value;

use crate::middleware::auth::get_cookie;

pub fn get_user_id_from_headers(headers: &HeaderMap, jwt_secret: &str) -> Option<i32> {
    let cookie = get_cookie(headers)?;
    let token = extract_cookie_value(&cookie, "token")?;
    let user_id = service::token::get_id(token, jwt_secret)?;

    Some(user_id)
}
