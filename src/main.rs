use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;

mod patients;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Read DATABASE_URL
    let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (use .env)");

    // Create pool
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("failed to connect to Postgres");

    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/patients", patients::router())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}