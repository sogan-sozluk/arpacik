use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use tower_http::cors::CorsLayer;

pub mod auth;
pub mod bin;
pub mod entry;
pub mod feed;
pub mod hello;
pub mod search;
pub mod statistics;
pub mod title;
pub mod today;
pub mod trends;

pub fn build(state: AppState) -> Router {
    let prefix = "/api/v1";

    let public = Router::new()
        .route("/hello", get(hello::hello_world))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/entries/:id", get(entry::get_entry))
        .route("/users/:id/entries", get(entry::get_user_entries))
        .route(
            "/titles/:name/entries",
            get(entry::get_title_entries_by_name),
        )
        .route("/today", get(today::today))
        .route("/trends", get(trends::trends))
        .route("/feed", get(feed::feed))
        .route("/search", get(search::search));

    let user = Router::new()
        .route("/auth/logout", post(auth::logout))
        .route("/entries", post(entry::create_entry))
        .route("/entries/:id", delete(entry::delete_entry))
        .route("/entries/:id/recover", patch(entry::recover_entry))
        .route("/entries/:id/soft-delete", delete(entry::soft_delete_entry))
        .route("/entries/:id", patch(entry::update_entry))
        .route("/entries/:id/favorite", post(entry::favorite_entry))
        .route("/entries/:id/unfavorite", post(entry::unfavorite_entry))
        .route("/entries/:id/vote/:rating", post(entry::vote_entry))
        .route("/entries/:id/unvote", post(entry::unvote_entry))
        .route("/self/bin", get(bin::get_user_bin))
        .route("/self/bin", delete(bin::empty_user_bin))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth::auth,
        ));

    let crew = Router::new()
        .route(
            "/entries/:id/to-title/:title_id",
            patch(entry::migrate_entry),
        )
        .route(
            "/titles/:id/set-visibility/:is_visible",
            patch(title::set_title_visibility),
        )
        .route("/statistics", get(statistics::statistics))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth::auth_crew,
        ));

    Router::new()
        .nest(prefix, public)
        .nest(prefix, user)
        .nest(prefix, crew)
        // TODO: Remove this once we have a proper frontend
        .layer(CorsLayer::permissive())
        .with_state(state)
}
