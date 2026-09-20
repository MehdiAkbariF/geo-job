use serde::Deserialize;
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