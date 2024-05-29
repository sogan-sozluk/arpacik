use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use service::{
    dto::{
        entry::{CreateEntryRequest, EntryDto},
        pagination::{PaginationRequest, PaginationResponse},
    },
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

pub async fn delete_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let cookie = get_cookie(&headers).ok_or((
        StatusCode::UNAUTHORIZED,
        Json(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response()),
    ))?;
    match service::entry::delete_entry(&state.conn, id, &cookie).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let error_response = e.into_error_response();
            Err((
                StatusCode::from_u16(error_response.code).unwrap(),
                Json(error_response),
            ))
        }
    }
}

pub async fn get_entry(
    state: State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<service::dto::entry::EntryDto>, (StatusCode, Json<ErrorResponse>)> {
    match service::entry::get_entry(&state.conn, id).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => {
            let error_response = e.into_error_response();
            Err((
                StatusCode::from_u16(error_response.code).unwrap(),
                Json(error_response),
            ))
        }
    }
}

pub async fn get_title_entries(
    state: State<AppState>,
    Path(title_id): Path<i32>,
    pagination: Query<PaginationRequest>,
) -> Result<Json<PaginationResponse<EntryDto>>, (StatusCode, Json<ErrorResponse>)> {
    let pagination = pagination.0;
    match service::entry::get_title_entries(&state.conn, title_id, pagination).await {
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
