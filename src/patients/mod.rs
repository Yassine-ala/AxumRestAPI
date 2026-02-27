use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::common::validation::email::{EmailValidationError, validate_email_opt};
use crate::common::error::{ApiError, SqlxResultExt};

pub fn domain_router() -> Router<AppState> {
    Router::new().route("/", get(list_patients).post(create_patient))
        .route("/{patient_id}", get(get_patient).put(update_patient).delete(delete_patient),
    )
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

fn map_email_err(e: EmailValidationError) -> ApiError {
    match e {
        EmailValidationError::Empty => {
            ApiError::BadRequest("email must be null or non-empty".into())
        }
        EmailValidationError::InvalidFormat => ApiError::BadRequest("invalid email format".into()),
    }
}

async fn create_patient(
    State(state): State<AppState>,
    Path(domain_id): Path<Uuid>,
    Json(dto): Json<CreatePatientDto>,
) -> Result<(StatusCode, Json<Patient>), ApiError> {
    validate_email_opt(&dto.email).map_err(map_email_err)?;

    let p = sqlx::query_as!(
        Patient,
        r#"
        INSERT INTO patients (domain_id, first_name, last_name, birth_date, email)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, first_name, last_name, birth_date, email, created_at, updated_at
        "#,
        domain_id,
        dto.first_name,
        dto.last_name,
        dto.birth_date,
        dto.email
    )
        .fetch_one(&state.db)
        .await
        .api_err()?;

    Ok((StatusCode::CREATED, Json(p)))
}

async fn get_patient(
    State(state): State<AppState>,
    Path((domain_id, patient_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Patient>, ApiError> {
    let p = sqlx::query_as!(
        Patient,
        r#"
        SELECT id, first_name, last_name, birth_date, email, created_at, updated_at
        FROM patients
        WHERE id = $1 AND domain_id = $2
        "#,
        patient_id,
        domain_id
    )
    .fetch_optional(&state.db)
    .await
    .api_err()?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(p))
}

async fn list_patients(
    State(state): State<AppState>,
    Path(domain_id): Path<Uuid>,
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
        WHERE domain_id = $1
          AND ($2 = '' OR first_name ILIKE $3 OR last_name ILIKE $3 OR email ILIKE $3)
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        domain_id,
        search,
        like
    )
    .fetch_all(&state.db)
    .await
    .api_err()?;

    Ok(Json(items))
}

async fn update_patient(
    State(state): State<AppState>,
    Path((domain_id, patient_id)): Path<(Uuid, Uuid)>,
    Json(dto): Json<UpdatePatientDto>,
) -> Result<Json<Patient>, ApiError> {
    if dto.email.is_some() {
        validate_email_opt(&dto.email).map_err(map_email_err)?;
    }

    let p = sqlx::query_as!(
        Patient,
        r#"
        UPDATE patients
        SET
          first_name = COALESCE($3, first_name),
          last_name  = COALESCE($4, last_name),
          birth_date = COALESCE($5, birth_date),
          email      = COALESCE($6, email),
          updated_at = now()
        WHERE id = $1 AND domain_id = $2
        RETURNING id, first_name, last_name, birth_date, email, created_at, updated_at
        "#,
        patient_id,
        domain_id,
        dto.first_name,
        dto.last_name,
        dto.birth_date,
        dto.email
    )
        .fetch_optional(&state.db)
        .await
        .api_err()?
        .ok_or(ApiError::NotFound)?;

    Ok(Json(p))
}

async fn delete_patient(
    State(state): State<AppState>,
    Path((domain_id, patient_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let res = sqlx::query!(
        "DELETE FROM patients WHERE id = $1 AND domain_id = $2",
        patient_id,
        domain_id
    )
        .execute(&state.db)
        .await
        .api_err()?;

    if res.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
