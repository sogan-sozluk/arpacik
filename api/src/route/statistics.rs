use axum::{extract::State, http::StatusCode, Json};
use service::dto::statistics::StatisticsResponse;

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    AppState,
};

pub async fn statistics(
    state: State<AppState>,
) -> Result<Json<StatisticsResponse>, (StatusCode, Json<ErrorBody>)> {
    match service::statistics::statistics(&state.conn).await {
        Ok(r) => Ok(Json(r)),
        Err(e) => Err(e.into_error_response()),
    }
}
