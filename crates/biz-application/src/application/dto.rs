use crate::candidate::CandidateProfileDto;
use biz_domain::application::Application;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SubmitApplicationCommand {
    #[schema(value_type = String, format = Uuid)]
    pub opportunity_id: Uuid,
    #[schema(value_type = Option<String>, format = Uuid)]
    pub resume_id: Option<Uuid>,
    pub cover_letter: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ChangeApplicationStatusCommand {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApplicationDto {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub opportunity_id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApplicationDossierDto {
    pub application: Application,
    pub candidate_profile: CandidateProfileDto,
}