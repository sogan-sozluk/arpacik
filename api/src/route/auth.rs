use axum::http::header::SET_COOKIE;
use axum::http::HeaderMap;
use axum::{extract::State, http::StatusCode, Json};
use service::dto::auth::{LoginRequest, RegisterRequest};
use service::error::{ErrorResponse, IntoErrorResponse};

use crate::AppState;

pub async fn register(
    state: State<AppState>,
    json_data: Json<RegisterRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    match service::auth::register(&state.conn, json_data.0).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            let error_response = e.into_error_response();
            Err((
                StatusCode::from_u16(error_response.code).unwrap(),
                Json(error_response),
            ))
        }
    }
}

pub async fn login(
    state: State<AppState>,
    json_data: Json<LoginRequest>,
) -> Result<(HeaderMap, StatusCode), (StatusCode, Json<ErrorResponse>)> {
    match service::auth::login(&state.conn, json_data.0).await {
        Ok(cookie) => {
            let mut headers = HeaderMap::new();
            headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());
            Ok((headers, StatusCode::OK))
        }
        Err(e) => {
            let error_response = e.into_error_response();
            Err((
                StatusCode::from_u16(error_response.code).unwrap(),
                Json(error_response),
            ))
        }
    }
}
