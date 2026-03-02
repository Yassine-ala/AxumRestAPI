use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;
use crate::common::error::{ApiError, SqlxResultExt};
use crate::domains::dto::AttributeConfigDto;
use crate::domains::models::{AttributeConfig, Domain, DomainRow};

pub(super) async fn fetch_domain(
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

pub(super) async fn insert_attribute_configs(
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