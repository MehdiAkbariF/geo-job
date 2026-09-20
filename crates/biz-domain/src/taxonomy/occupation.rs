use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Occupation {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub created_at: DateTime<Utc>,
}