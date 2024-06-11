use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use service::dto::title::TitleDto;

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    AppState,
};

pub async fn set_title_visibility(
    state: State<AppState>,
    Path((id, is_visible)): Path<(i32, bool)>,
) -> Result<Json<TitleDto>, (StatusCode, Json<ErrorBody>)> {
    match service::title::set_title_visibility(&state.conn, id, is_visible).await {
        Ok(title) => Ok(Json(title)),
        Err(e) => Err(e.into_error_response()),
    }
}
