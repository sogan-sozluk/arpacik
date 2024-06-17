use crate::error::{ErrorBody, IntoErrorResponse};
use crate::traits::HeaderToken;
use crate::AppState;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::{extract::State, http::StatusCode, Json};
use service::dto::auth::{LoginRequest, LoginResponse, RegisterRequest};
use service::Error;

pub async fn register(
    state: State<AppState>,
    json_data: Json<RegisterRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    match service::auth::register(&state.conn, json_data.0).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn login(
    state: State<AppState>,
    json_data: Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), (StatusCode, Json<ErrorBody>)> {
    match service::auth::login(&state.conn, json_data.0).await {
        Ok(cookie) => {
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());
            Ok((
                headers,
                Json(LoginResponse {
                    token: cookie.value,
                }),
            ))
        }
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn logout(
    state: State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, StatusCode), (StatusCode, Json<ErrorBody>)> {
    let token = headers
        .token(state.auth_from)
        .ok_or(Error::Unauthorized("Geçersiz token".to_string()).into_error_response())?;
    match service::auth::logout(&state.conn, &token).await {
        Ok(cookie) => {
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());
            Ok((headers, StatusCode::OK))
        }
        Err(e) => Err(e.into_error_response()),
    }
}
