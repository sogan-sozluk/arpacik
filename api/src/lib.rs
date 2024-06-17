use axum::Router;
use migration::{Migrator, MigratorTrait};
use service::{
    auth::AuthHeader,
    sea_orm::{ConnectOptions, Database, DatabaseConnection},
};
use std::env;

mod error;
mod helper;
mod middleware;
mod route;
mod traits;

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
    jwt_secret: String,
    auth_from: AuthHeader,
}

#[tokio::main]
async fn start() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{host}:{port}");

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_schema = env::var("DATABASE_SCHEMA").expect("DATABASE_SCHEMA is not set in .env file");
    let mut opt = ConnectOptions::new(db_url);
    opt.set_schema_search_path(db_schema);

    let conn = Database::connect(opt)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET is not set in .env file");

    let auth_from = match env::var("AUTH_FROM")
        .expect("AUTH_FROM is not set in .env file")
        .as_str()
    {
        "cookie" => AuthHeader::Cookie,
        "authorization" => AuthHeader::Authorization,
        _ => AuthHeader::Cookie,
    };

    let state = AppState {
        conn,
        jwt_secret,
        auth_from,
    };

    let router = route::build(state);
    let app = Router::new().nest("/", router);
    let listener = tokio::net::TcpListener::bind(&server_url).await.unwrap();

    axum::serve(listener, app).await?;

    Ok(())
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}
