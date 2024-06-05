use axum::{extract::State, http::StatusCode, Json};
use service::dto::entry::EntryDto;

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    AppState,
};

pub async fn feed(
    state: State<AppState>,
) -> Result<Json<Vec<EntryDto>>, (StatusCode, Json<ErrorBody>)> {
    match service::feed::feed(&state.conn).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}
