use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateReportCommand {
    #[schema(value_type = String, format = Uuid)]
    pub opportunity_id: Uuid,
    pub reason: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SubmitVerificationCommand {
    #[schema(value_type = String, format = Uuid)]
    pub company_id: Uuid,
    pub evidence_storage_keys: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ReviewVerificationCommand {
    pub status: String, // "verified" or "rejected"
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompanyVerificationDto {
    pub id: Uuid,
    pub company_id: Uuid,
    pub status: String,
    pub evidence_storage_keys: Vec<String>,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
}