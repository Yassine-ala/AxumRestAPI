use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_patient).get(list_patients))
        .route("/{id}", get(get_patient).put(update_patient).delete(delete_patient))
}

#[derive(Debug, Serialize)]
struct Patient {
    id: Uuid,
    first_name: String,
    last_name: String,
    birth_date: NaiveDate,
    email: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreatePatientDto {
    first_name: String,
    last_name: String,
    birth_date: NaiveDate,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdatePatientDto {
    first_name: Option<String>,
    last_name: Option<String>,
    birth_date: Option<NaiveDate>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    search: Option<String>,
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("conflict")]
    Conflict,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("database error")]
    Db(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::NotFound => StatusCode::NOT_FOUND.into_response(),
            ApiError::Conflict => StatusCode::CONFLICT.into_response(),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            ApiError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

fn map_sqlx_error(e: sqlx::Error) -> ApiError {
    // Unique violation -> 409
    if let sqlx::Error::Database(db_err) = &e {
        // 23505 = unique_violation (Postgres SQLSTATE)
        if db_err.code().as_deref() == Some("23505") {
            return ApiError::Conflict;
        }
    }
    ApiError::Db(e)
}

async fn create_patient(
    State(state): State<AppState>,
    Json(dto): Json<CreatePatientDto>,
) -> Result<Json<Patient>, ApiError> {
    let p = sqlx::query_as!(
        Patient,
        r#"
        INSERT INTO patients (first_name, last_name, birth_date, email)
        VALUES ($1, $2, $3, $4)
        RETURNING id, first_name, last_name, birth_date, email, created_at, updated_at
        "#,
        dto.first_name,
        dto.last_name,
        dto.birth_date,
        dto.email
    )
        .fetch_one(&state.db)
        .await
        .map_err(map_sqlx_error)?;

    Ok(Json(p))
}

async fn get_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Patient>, ApiError> {
    let p = sqlx::query_as!(
        Patient,
        r#"
        SELECT id, first_name, last_name, birth_date, email, created_at, updated_at
        FROM patients
        WHERE id = $1
        "#,
        id
    )
        .fetch_optional(&state.db)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(p))
}

async fn list_patients(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Patient>>, ApiError> {
    // Simple search by name/email. Expand later.
    let search = q.search.unwrap_or_default();
    let like = format!("%{}%", search);

    let items = sqlx::query_as!(
        Patient,
        r#"
        SELECT id, first_name, last_name, birth_date, email, created_at, updated_at
        FROM patients
        WHERE ($1 = '' OR first_name ILIKE $2 OR last_name ILIKE $2 OR email ILIKE $2)
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        search,
        like
    )
        .fetch_all(&state.db)
        .await
        .map_err(map_sqlx_error)?;

    Ok(Json(items))
}

async fn update_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdatePatientDto>,
) -> Result<Json<Patient>, ApiError> {
    // Update only provided fields (COALESCE trick)
    let p = sqlx::query_as!(
        Patient,
        r#"
        UPDATE patients
        SET
          first_name = COALESCE($2, first_name),
          last_name  = COALESCE($3, last_name),
          birth_date = COALESCE($4, birth_date),
          email      = COALESCE($5, email),
          updated_at = now()
        WHERE id = $1
        RETURNING id, first_name, last_name, birth_date, email, created_at, updated_at
        "#,
        id,
        dto.first_name,
        dto.last_name,
        dto.birth_date,
        dto.email
    )
        .fetch_optional(&state.db)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(p))
}

async fn delete_patient(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query!("DELETE FROM patients WHERE id = $1", id)
        .execute(&state.db)
        .await
        .map_err(map_sqlx_error)?;

    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}