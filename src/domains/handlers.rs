use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use uuid::Uuid;
use crate::AppState;
use crate::common::error::{ApiError, SqlxResultExt};
use crate::domains::db::{fetch_domain, insert_attribute_configs};
use crate::domains::dto::{CreateDomainDto, UpdateDomainDto};
use crate::domains::models::{Domain, DomainRow};
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

async fn create_domain(
    State(state): State<AppState>,
    Json(dto): Json<CreateDomainDto>,
) -> Result<(StatusCode, Json<Domain>), ApiError> {
    let mut tx = state.db.begin().await.api_err()?;

    let domain = sqlx::query_as!(
        DomainRow,
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
        .fetch_one(&mut *tx)
        .await
        .api_err()?;

    if let Some(configs) = dto.attribute_configs {
        insert_attribute_configs(&mut tx, domain.id, configs).await?;
    }

    tx.commit().await.api_err()?;

    let domain = fetch_domain(&state.db, domain.id).await?;
    Ok((StatusCode::CREATED, Json(domain)))
}

async fn get_domain(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Domain>, ApiError> {
    Ok(Json(fetch_domain(&state.db, id).await?))
}

async fn list_domains(State(state): State<AppState>) -> Result<Json<Vec<DomainRow>>, ApiError> {
    let domain_rows = sqlx::query_as!(
        DomainRow,
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

    Ok(Json(domain_rows))
}

async fn update_domains(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateDomainDto>,
) -> Result<Json<Domain>, ApiError> {
    let mut tx = state.db.begin().await.api_err()?;

    sqlx::query_as!(
        DomainRow,
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
        .fetch_optional(&mut *tx)
        .await
        .api_err()?
        .ok_or(ApiError::NotFound)?;

    if let Some(configs) = dto.attribute_configs {
        // replace-all
        sqlx::query!("DELETE FROM attribute_configs WHERE domain_id = $1", id)
            .execute(&mut *tx)
            .await
            .api_err()?;

        insert_attribute_configs(&mut tx, id, configs).await?;
    }

    tx.commit().await.api_err()?;

    let domain = fetch_domain(&state.db, id).await?;
    Ok(Json(domain))
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