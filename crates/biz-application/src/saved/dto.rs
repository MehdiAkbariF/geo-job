use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct SaveSearchCommand {
    pub title: String,
    pub criteria: serde_json::Value,
}