use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use service::{
    dto::{
        pagination::{PaginationQuery, PaginationResponse},
        today::TodayTitleDto,
    },
    error::{ErrorResponse, IntoErrorResponse},
};

use crate::AppState;

pub async fn today(
    state: State<AppState>,
    query: Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<TodayTitleDto>>, (StatusCode, Json<ErrorResponse>)> {
    match service::today::today(&state.conn, query.0).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => {
            let error_response = e.into_error_response();
            Err((
                StatusCode::from_u16(error_response.code).unwrap(),
                Json(error_response),
            ))
        }
    }
}
