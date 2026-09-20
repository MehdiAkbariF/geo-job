use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateReportCommand {
    pub opportunity_id: Uuid,
    pub reason: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitVerificationCommand {
    pub company_id: Uuid,
    pub evidence_storage_keys: Vec<String>,
}