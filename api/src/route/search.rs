use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use service::dto::search::{SearchItem, SearchQuery};

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    AppState,
};

pub async fn search(
    state: State<AppState>,
    request: Query<SearchQuery>,
) -> Result<Json<Vec<SearchItem>>, (StatusCode, Json<ErrorBody>)> {
    match service::search::search(&state.conn, request.0).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}
