use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub action_url: Option<String>,
    pub notification_type: String,
    pub is_read: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}