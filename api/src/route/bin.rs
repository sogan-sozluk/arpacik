use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use service::{
    dto::{
        entry::EntryDto,
        pagination::{PaginationQuery, PaginationResponse},
    },
    Error,
};

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    helper::get_user_id_from_headers,
    AppState,
};

pub async fn get_user_bin(
    state: State<AppState>,
    headers: HeaderMap,
    query: Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<EntryDto>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::bin::get_user_bin(&state.conn, user_id, query.0).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn empty_user_bin(
    state: State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::bin::empty_user_bin(&state.conn, user_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}
