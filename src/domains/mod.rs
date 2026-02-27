use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::common::error::{ApiError, SqlxResultExt};
use crate::patients::domain_router;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_domain).get(list_domains))
        .route(
            "/{id}",
            get(get_domain).put(update_domains).delete(delete_domain),
        )
        .nest("/{id}/patients", domain_router())
}

#[derive(Debug, Serialize)]
struct Domain {
    id: Uuid,
    label: String,
    country: String,
    language: String,
    max_search_identity: i16,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateDomainDto {
    label: String,
    country: String,
    language: String,
    max_search_identity: i16,
}

#[derive(Debug, Deserialize)]
struct UpdateDomainDto {
    label: Option<String>,
    country: Option<String>,
    language: Option<String>,
    max_search_identity: Option<i16>,
}

async fn create_domain(
    State(state): State<AppState>,
    Json(dto): Json<CreateDomainDto>,
) -> Result<(StatusCode, Json<Domain>), ApiError> {
    let p = sqlx::query_as!(
        Domain,
        r#"
        INSERT INTO domains (label, country, language, max_search_identity)
        VALUES ($1, $2, $3, $4)
        RETURNING id, label, country, language, max_search_identity, created_at, updated_at
        "#,
        dto.label,
        dto.country,
        dto.language,
        dto.max_search_identity
    )
    .fetch_one(&state.db)
    .await
    .api_err()?;

    Ok((StatusCode::CREATED, Json(p)))
}

async fn get_domain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Domain>, ApiError> {
    let p = sqlx::query_as!(
        Domain,
        r#"
        SELECT id, label, country, language, max_search_identity, created_at, updated_at
        FROM domains
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&state.db)
    .await
    .api_err()?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(p))
}

async fn list_domains(State(state): State<AppState>) -> Result<Json<Vec<Domain>>, ApiError> {
    let items = sqlx::query_as!(
        Domain,
        r#"
        SELECT id, label, country, language, max_search_identity, created_at, updated_at
        FROM domains
        ORDER BY label DESC
        LIMIT 50
        "#
    )
    .fetch_all(&state.db)
    .await
    .api_err()?;

    Ok(Json(items))
}

async fn update_domains(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateDomainDto>,
) -> Result<Json<Domain>, ApiError> {
    let p = sqlx::query_as!(
        Domain,
        r#"
        UPDATE domains
        SET
          label = COALESCE($2, label),
          country  = COALESCE($3, country),
          language = COALESCE($4, language),
          max_search_identity = COALESCE($5, max_search_identity),
          updated_at = now()
        WHERE id = $1
        RETURNING id, label, country, language, max_search_identity, created_at, updated_at
        "#,
        id,
        dto.label,
        dto.country,
        dto.language,
        dto.max_search_identity
    )
    .fetch_optional(&state.db)
    .await
    .api_err()?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(p))
}

async fn delete_domain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query!("DELETE FROM domains WHERE id = $1", id)
        .execute(&state.db)
        .await
        .api_err()?;

    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
