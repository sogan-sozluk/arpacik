use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use service::{
    dto::entry::CreateEntryRequest,
    error::{ErrorResponse, IntoErrorResponse},
    Error,
};

use crate::{middleware::auth::get_cookie, AppState};

pub async fn create_entry(
    state: State<AppState>,
    headers: HeaderMap,
    json_data: Json<CreateEntryRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let cookie = get_cookie(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response()),
    ))?;
    match service::entry::create_entry(&state.conn, json_data.0, &cookie).await {
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
