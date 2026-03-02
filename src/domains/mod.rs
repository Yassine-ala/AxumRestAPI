use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};
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
    attribute_configs: Vec<AttributeConfig>,
}

#[derive(Debug, Serialize)]
struct DomainRow {
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
    attribute_configs: Option<Vec<AttributeConfigDto>>,
}

#[derive(Debug, Deserialize)]
struct UpdateDomainDto {
    label: Option<String>,
    country: Option<String>,
    language: Option<String>,
    max_search_identity: Option<i16>,
    attribute_configs: Option<Vec<AttributeConfigDto>>,
}

#[derive(Debug, Deserialize)]
struct AttributeConfigDto {
    attribute_id: Uuid,
    search_weight: i16,
    search_index: i16,
    appears_in_banner: bool,
    mandatory_in_form: bool,
    appears_in_search: bool,
    mandatory_in_search: bool,
}

#[derive(Debug, Serialize)]
struct AttributeConfig {
    attribute_id: Uuid,
    search_weight: i16,
    search_index: i16,
    appears_in_banner: bool,
    mandatory_in_form: bool,
    appears_in_search: bool,
    mandatory_in_search: bool,
}

async fn fetch_domain(
    db: &sqlx::PgPool,
    id: Uuid,
) -> Result<Domain, ApiError> {
    let row = sqlx::query_as!(
        DomainRow,
        r#"
        SELECT id, label, country, language, max_search_identity, created_at, updated_at
        FROM domains
        WHERE id = $1
        "#,
        id
    )
        .fetch_optional(db)
        .await
        .api_err()?
        .ok_or(ApiError::NotFound)?;

    let configs = sqlx::query_as!(
        AttributeConfig,
        r#"
        SELECT
          attribute_id,
          search_weight,
          search_index,
          appears_in_banner,
          mandatory_in_form,
          appears_in_search,
          mandatory_in_search
        FROM attribute_configs
        WHERE domain_id = $1
        ORDER BY search_index
        "#,
        id
    )
        .fetch_all(db)
        .await
        .api_err()?; // returns [] if none

    Ok(Domain {
        id: row.id,
        label: row.label,
        country: row.country,
        language: row.language,
        max_search_identity: row.max_search_identity,
        created_at: row.created_at,
        updated_at: row.updated_at,
        attribute_configs: configs,
    })
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

async fn insert_attribute_configs(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    domain_id: Uuid,
    configs: Vec<AttributeConfigDto>,
) -> Result<(), ApiError> {
    if configs.is_empty() {
        return Ok(());
    }

    let mut qb = QueryBuilder::<Postgres>::new(
        r#"
        INSERT INTO attribute_configs
          (domain_id, attribute_id, search_weight, search_index, mandatory_in_form, appears_in_banner, appears_in_search, mandatory_in_search)
        "#,
    );

    qb.push_values(configs, |mut b, c| {
        b.push_bind(domain_id)
            .push_bind(c.attribute_id)
            .push_bind(c.search_weight)
            .push_bind(c.search_index)
            .push_bind(c.mandatory_in_form)
            .push_bind(c.appears_in_banner)
            .push_bind(c.appears_in_search)
            .push_bind(c.mandatory_in_search);
    });

    qb.build().execute(&mut **tx).await.api_err()?;

    Ok(())
}
