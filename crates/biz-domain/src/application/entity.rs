use super::lifecycle::ApplicationStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Application {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub opportunity_id: Uuid,
    pub resume_id: Option<Uuid>,
    pub cover_letter: Option<String>,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewApplication {
    pub candidate_id: Uuid,
    pub opportunity_id: Uuid,
    pub resume_id: Option<Uuid>,
    pub cover_letter: Option<String>,
}