use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// AuditLog is immutable and append-only (Section 47 & 63 of MASTER PROMPT 03).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}