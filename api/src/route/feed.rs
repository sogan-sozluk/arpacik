use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use service::dto::entry::EntryDto;

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    helper::get_user_id_from_headers,
    AppState,
};

pub async fn feed(
    state: State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<EntryDto>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret);
    match service::feed::feed(&state.conn, user_id).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}
