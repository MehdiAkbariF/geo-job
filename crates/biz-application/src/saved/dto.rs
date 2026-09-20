use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SaveSearchCommand {
    pub title: String,
    pub criteria: serde_json::Value,
}