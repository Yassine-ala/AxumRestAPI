use chrono::{DateTime, Utc};
use serde::{Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub(crate) struct Domain {
    pub(crate) id: Uuid,
    pub(crate) label: String,
    pub(crate) country: String,
    pub(crate) language: String,
    pub(crate) max_search_identity: i16,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) attribute_configs: Vec<AttributeConfig>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DomainRow {
    pub(crate) id: Uuid,
    pub(crate) label: String,
    pub(crate) country: String,
    pub(crate) language: String,
    pub(crate) max_search_identity: i16,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub(crate) struct AttributeConfig {
    pub(crate) attribute_id: Uuid,
    pub(crate) search_weight: i16,
    pub(crate) search_index: i16,
    pub(crate) appears_in_banner: bool,
    pub(crate) mandatory_in_form: bool,
    pub(crate) appears_in_search: bool,
    pub(crate) mandatory_in_search: bool,
}