use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitApplicationCommand {
    pub opportunity_id: Uuid,
    pub resume_id: Option<Uuid>,
    pub cover_letter: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangeApplicationStatusCommand {
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationDto {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub opportunity_id: Uuid,
    pub status: String,
}