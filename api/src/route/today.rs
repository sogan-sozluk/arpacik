use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use service::dto::{
    pagination::{PaginationQuery, PaginationResponse},
    today::TodayTitleDto,
};

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    AppState,
};

pub async fn today(
    state: State<AppState>,
    query: Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<TodayTitleDto>>, (StatusCode, Json<ErrorBody>)> {
    match service::today::today(&state.conn, query.0).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}
