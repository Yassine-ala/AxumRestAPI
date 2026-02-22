pub mod patients;

use axum::Router;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/patients", patients::router())
        .with_state(state)
}