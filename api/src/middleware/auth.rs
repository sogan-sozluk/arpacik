use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use service::auth::{authorize, Role};

use crate::{traits::HeaderToken, AppState};

pub async fn auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match headers.token(state.auth_from) {
        Some(token) if authorize(&token, &state.jwt_secret, Role::User) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn _auth_moderator(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match headers.token(state.auth_from) {
        Some(token) if authorize(&token, &state.jwt_secret, Role::Moderator) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn _auth_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match headers.token(state.auth_from) {
        Some(token) if authorize(&token, &state.jwt_secret, Role::Admin) => {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn auth_crew(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    match headers.token(state.auth_from) {
        Some(token)
            if authorize(&token, &state.jwt_secret, Role::Moderator)
                || authorize(&token, &state.jwt_secret, Role::Admin) =>
        {
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
