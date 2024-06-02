use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};

pub mod auth;
pub mod entry;
pub mod hello;
pub mod today;

pub fn build(state: AppState) -> Router {
    let api_router = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/entries", post(entry::create_entry))
        .route("/entries/:id", delete(entry::delete_entry))
        .route("/entries/:id/recover", patch(entry::recover_entry))
        .route("/entries/:id/soft-delete", delete(entry::soft_delete_entry))
        .route("/entries/:id", patch(entry::update_entry))
        .route(
            "/entries/:id/to-title/:title_id",
            patch(entry::migrate_entry),
        )
        .route_layer(middleware::from_fn(crate::middleware::auth::auth))
        .route("/hello", get(hello::hello_world))
        .route("/entries/:id", get(entry::get_entry))
        .route("/users/:id/entries", get(entry::get_user_entries))
        .route("/titles/:id/entries", get(entry::get_title_entries))
        .route("/today", get(today::today))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .with_state(state.clone());

    Router::new().nest("/api/v1", api_router).with_state(state)
}
