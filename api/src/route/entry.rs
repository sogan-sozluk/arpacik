use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use service::{
    dto::{
        entry::{
            CreateEntryRequest, CreateEntryResponse, EntryDto, GetTitleEntriesQuery,
            UpdateEntryRequest,
        },
        pagination::{PaginationQuery, PaginationResponse},
    },
    Error,
};

use crate::{
    error::{ErrorBody, IntoErrorResponse},
    helper::get_user_id_from_headers,
    AppState,
};

pub async fn create_entry(
    state: State<AppState>,
    headers: HeaderMap,
    json_data: Json<CreateEntryRequest>,
) -> Result<Json<CreateEntryResponse>, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::create_entry(&state.conn, user_id, json_data.0).await {
        Ok(r) => Ok(Json(r)),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn delete_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::delete_entry(&state.conn, user_id, id, false).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn soft_delete_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::delete_entry(&state.conn, user_id, id, true).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn recover_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::recover_entry(&state.conn, user_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn get_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<Json<service::dto::entry::EntryDto>, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret);
    match service::entry::get_entry(&state.conn, id, user_id).await {
        Ok(entry) => Ok(Json(entry)),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn get_title_entries_by_name(
    state: State<AppState>,
    headers: HeaderMap,
    Path(name): Path<String>,
    query: Query<GetTitleEntriesQuery>,
) -> Result<Json<PaginationResponse<EntryDto>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret);
    match service::entry::get_title_entries_by_name(&state.conn, &name, query.0, user_id).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn get_user_entries(
    state: State<AppState>,
    headers: HeaderMap,
    Path(author_id): Path<i32>,
    query: Query<PaginationQuery>,
) -> Result<Json<PaginationResponse<EntryDto>>, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret);
    match service::entry::get_user_entries(&state.conn, author_id, query.0, user_id).await {
        Ok(entries) => Ok(Json(entries)),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn update_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
    json_data: Json<UpdateEntryRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::update_entry(&state.conn, user_id, id, json_data.0).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn migrate_entry(
    state: State<AppState>,
    Path((id, title_id)): Path<(i32, i32)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    match service::entry::migrate_entry(&state.conn, id, title_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn favorite_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::favorite_entry(&state.conn, user_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn unfavorite_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::unfavorite_entry(&state.conn, user_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn upvote(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::upvote(&state.conn, user_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn downvote(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::downvote(&state.conn, user_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}

pub async fn unvote_entry(
    state: State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorBody>)> {
    let user_id = get_user_id_from_headers(&headers, state.auth_from, &state.jwt_secret)
        .ok_or(Error::Unauthorized("Geçersiz çerez".to_string()).into_error_response())?;
    match service::entry::unvote(&state.conn, user_id, id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into_error_response()),
    }
}
