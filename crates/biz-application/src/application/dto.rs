use crate::candidate::CandidateProfileDto;
use biz_domain::application::Application;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SubmitApplicationCommand {
    #[serde(default)]
    #[schema(value_type = Option<String>, format = Uuid)]
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

/// Enriched Applicant summary for Employer ATS Pipeline
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApplicantSummaryDto {
    pub application_id: Uuid,
    pub candidate_id: Uuid,
    pub candidate_name: String,
    pub candidate_headline: Option<String>,
    pub candidate_city: Option<String>,
    pub status: String,
    pub match_score: Option<u8>,
    pub cover_letter: Option<String>,
    pub resume_id: Option<Uuid>,
    pub resume_filename: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ApplicationDossierDto {
    pub application: Application,
    pub candidate_profile: CandidateProfileDto,
    pub match_score: Option<u8>,
    pub commute_distance_meters: Option<f64>,
}