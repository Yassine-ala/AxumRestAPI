pub mod patients;
pub mod domains;
pub mod attributes;
pub mod common;

use axum::Router;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/domains", domains::router())
        .nest("/attributes", attributes::router())
        .with_state(state)
}