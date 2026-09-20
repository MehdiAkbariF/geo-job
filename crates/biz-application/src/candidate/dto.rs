use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateProfileCommand {
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub preferred_city: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddExperienceCommand {
    pub title: String,
    pub company_name: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_current: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExperienceDto {
    pub id: Uuid,
    pub title: String,
    pub company_name: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_current: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CandidateProfileDto {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub preferred_city: Option<String>,
    pub skills: Vec<Uuid>,
    pub experiences: Vec<ExperienceDto>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SetSkillsCommand {
    pub skill_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CandidatePreferencesDto {
    pub preferred_workplace_types: Vec<String>,
    pub preferred_opportunity_types: Vec<String>,
    pub expected_salary_min: Option<Decimal>,
    pub salary_currency: String,
    pub remote_only: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TrackedApplicationDto {
    pub application_id: Uuid,
    pub opportunity_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}