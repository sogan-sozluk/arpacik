use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub mod auth;
pub mod hello;

pub fn build(state: AppState) -> Router {
    let api_router = Router::new()
        .route("/hello", get(hello::hello_world))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .with_state(state.clone());

    Router::new().nest("/api/v1", api_router).with_state(state)
}
