use axum::http::HeaderMap;
use service::cookie::cookie_value;

use crate::traits::HeadersCookie;

pub fn get_user_id_from_headers(headers: &HeaderMap, jwt_secret: &str) -> Option<i32> {
    let cookie = headers.cookie()?;
    let token = cookie_value(&cookie, "token")?;
    let user_id = service::token::get_id(token, jwt_secret)?;

    Some(user_id)
}
