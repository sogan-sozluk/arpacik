use axum::http::HeaderMap;
use service::auth::AuthHeader;

use crate::traits::HeaderToken;

pub fn get_user_id_from_headers(
    headers: &HeaderMap,
    auth_from: AuthHeader,
    jwt_secret: &str,
) -> Option<i32> {
    let token = headers.token(auth_from)?;
    let user_id = service::token::get_id(&token, jwt_secret)?;

    Some(user_id)
}
