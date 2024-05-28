use axum::{extract::State, http::StatusCode, Json};
use service::dto::auth::RegisterRequest;
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
