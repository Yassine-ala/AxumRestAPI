use crate::common::error::SqlxResultExt;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::common::error::{ApiError};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_attribute).get(list_attributes))
        .route(
            "/{id}",
            get(get_attribute).put(update_attribute).delete(delete_attribute),
        )
}

#[derive(Debug, Serialize)]
struct Attribute {
    id: Uuid,
    key: String,
    label: String,
    description: Option<String>,
    data_type: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateAttributeDto {
    key: String,
    label: String,
    description: Option<String>,
    data_type: String,
}

#[derive(Debug, Deserialize)]
struct UpdateAttributeDto {
    key: Option<String>,
    label: Option<String>,
    description: Option<String>,
    data_type: Option<String>,
}

async fn create_attribute(
    State(state): State<AppState>,
    Json(dto): Json<CreateAttributeDto>,
) -> Result<(StatusCode, Json<Attribute>), ApiError> {
    let a = sqlx::query_as!(
        Attribute,
        r#"
        INSERT INTO attributes (key, label, description, data_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id, key, label, description, data_type, created_at, updated_at
        "#,
        dto.key,
        dto.label,
        dto.description,
        dto.data_type,
    )
        .fetch_one(&state.db)
        .await
        .api_err()?;

    Ok((StatusCode::CREATED, Json(a)))
}

async fn get_attribute(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Attribute>, ApiError> {
    let a = sqlx::query_as!(
        Attribute,
        r#"
        SELECT id, key, label, description, data_type, created_at, updated_at
        FROM attributes
        WHERE id = $1
        "#,
        id
    )
        .fetch_optional(&state.db)
        .await
        .api_err()?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(a))
}

async fn list_attributes(State(state): State<AppState>) -> Result<Json<Vec<Attribute>>, ApiError> {
    let items = sqlx::query_as!(
        Attribute,
        r#"
        SELECT id, key, label, description, data_type, created_at, updated_at
        FROM attributes
        ORDER BY key
        LIMIT 200
        "#
    )
        .fetch_all(&state.db)
        .await
        .api_err()?;

    Ok(Json(items))
}

async fn update_attribute(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateAttributeDto>,
) -> Result<Json<Attribute>, ApiError> {
    let a = sqlx::query_as!(
        Attribute,
        r#"
        UPDATE attributes
        SET
          key = COALESCE($2, key),
          label = COALESCE($3, label),
          description = COALESCE($4, description),
          data_type = COALESCE($5, data_type),
          updated_at = now()
        WHERE id = $1
        RETURNING id, key, label, description, data_type, created_at, updated_at
        "#,
        id,
        dto.key,
        dto.label,
        dto.description,
        dto.data_type
    )
        .fetch_optional(&state.db)
        .await
        .api_err()?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(a))
}

async fn delete_attribute(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query!("DELETE FROM attributes WHERE id = $1", id)
        .execute(&state.db)
        .await
        .api_err()?;

    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}