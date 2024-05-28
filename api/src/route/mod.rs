use crate::AppState;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub mod auth;
pub mod entry;
pub mod hello;

pub fn build(state: AppState) -> Router {
    let api_router = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/entry", post(entry::create_entry))
        .route_layer(middleware::from_fn(crate::middleware::auth::auth))
        .route("/hello", get(hello::hello_world))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .with_state(state.clone());

    Router::new().nest("/api/v1", api_router).with_state(state)
}
