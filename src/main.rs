use axum_api::{app, AppState}; // assuming you create lib.rs as shown below
use sqlx::PgPool;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let state = AppState { db };

    let app = app(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}