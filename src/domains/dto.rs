use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub(crate) struct CreateDomainDto {
    pub(crate) label: String,
    pub(crate) country: String,
    pub(crate) language: String,
    pub(crate) max_search_identity: i16,
    pub(crate) attribute_configs: Option<Vec<AttributeConfigDto>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateDomainDto {
    pub(crate) label: Option<String>,
    pub(crate) country: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) max_search_identity: Option<i16>,
    pub(crate) attribute_configs: Option<Vec<AttributeConfigDto>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AttributeConfigDto {
    pub(crate) attribute_id: Uuid,
    pub(crate) search_weight: i16,
    pub(crate) search_index: i16,
    pub(crate) appears_in_banner: bool,
    pub(crate) mandatory_in_form: bool,
    pub(crate) appears_in_search: bool,
    pub(crate) mandatory_in_search: bool,
}