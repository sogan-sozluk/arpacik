use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use service::dto::{
    pagination::{PaginationQuery, PaginationResponse},
    trends::TrendTitleDto,
};

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    AppState,
};

pub async fn trends(
    state: State<AppState>,
    query: Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<TrendTitleDto>>, (StatusCode, Json<ErrorBody>)> {
    match service::trends::trends(&state.conn, query.0).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}
