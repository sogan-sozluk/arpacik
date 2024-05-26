use axum::{extract::State, http::StatusCode};

use crate::AppState;

pub async fn hello_world(state: State<AppState>) -> (StatusCode, &'static str) {
    state.conn.ping().await.unwrap();
    (StatusCode::OK, "Hello, World!")
}
