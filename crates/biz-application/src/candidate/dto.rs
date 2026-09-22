use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateProfileCommand {
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub preferred_city: Option<String>,
    pub residence_location_id: Option<Uuid>,
    pub preferred_commute_radius_meters: Option<i32>,
    pub show_exact_location_to_employers: Option<bool>,
    pub is_foreign_national: Option<bool>,
    pub nationality_country_code: Option<String>,
    pub has_disability: Option<bool>,
    pub disability_type: Option<String>,
    pub gender: Option<String>,
    pub military_service_status: Option<String>,
    pub marital_status: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub preferred_category_ids: Option<Vec<Uuid>>,
    pub linkedin_url: Option<String>,
    pub github_url: Option<String>,
    pub website_url: Option<String>,
    pub audio_intro_storage_key: Option<String>,
    pub job_search_status: Option<String>,
    pub awards: Option<serde_json::Value>,
    pub certifications: Option<serde_json::Value>,
    pub academic_projects: Option<serde_json::Value>,
    pub publications: Option<serde_json::Value>,
    pub volunteering: Option<serde_json::Value>,
    pub portfolio_items: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddExperienceCommand {
    pub title: String,
    pub company_name: String,
    pub activity_field: Option<String>,
    pub seniority_level: Option<String>,
    pub company_industry: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub start_month: Option<i16>,
    pub start_year: Option<i32>,
    pub end_month: Option<i16>,
    pub end_year: Option<i32>,
    pub is_current: bool,
    pub achievements: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExperienceDto {
    pub id: Uuid,
    pub title: String,
    pub company_name: String,
    pub activity_field: Option<String>,
    pub seniority_level: Option<String>,
    pub company_industry: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub start_month: Option<i16>,
    pub start_year: Option<i32>,
    pub end_month: Option<i16>,
    pub end_year: Option<i32>,
    pub is_current: bool,
    pub achievements: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddEducationCommand {
    pub institution: String,
    pub degree_level: String,
    pub field_of_study: Option<String>,
    pub gpa: Option<Decimal>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EducationDto {
    pub id: Uuid,
    pub institution: String,
    pub degree_level: String,
    pub field_of_study: Option<String>,
    pub gpa: Option<Decimal>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddLanguageCommand {
    pub language_name: String,
    pub proficiency_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LanguageDto {
    pub id: Uuid,
    pub language_name: String,
    pub proficiency_level: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddReferenceCommand {
    pub full_name: String,
    pub organization_name: String,
    pub job_title: String,
    pub relationship_type: Option<String>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub is_still_colleagues: bool,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReferenceDto {
    pub id: Uuid,
    pub full_name: String,
    pub organization_name: String,
    pub job_title: String,
    pub relationship_type: Option<String>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub is_still_colleagues: bool,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddResumeCommand {
    pub storage_key: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ResumeDto {
    pub id: Uuid,
    pub storage_key: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SkillDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CandidateProfileDto {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub preferred_city: Option<String>,
    pub residence_location_id: Option<Uuid>,
    pub preferred_commute_radius_meters: Option<i32>,
    pub show_exact_location_to_employers: bool,
    pub is_foreign_national: bool,
    pub nationality_country_code: Option<String>,
    pub has_disability: bool,
    pub disability_type: Option<String>,
    pub gender: Option<String>,
    pub military_service_status: Option<String>,
    pub marital_status: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub preferred_category_ids: Vec<Uuid>,
    pub linkedin_url: Option<String>,
    pub github_url: Option<String>,
    pub website_url: Option<String>,
    pub audio_intro_storage_key: Option<String>,
    pub job_search_status: String,
    pub awards: serde_json::Value,
    pub certifications: serde_json::Value,
    pub academic_projects: serde_json::Value,
    pub publications: serde_json::Value,
    pub volunteering: serde_json::Value,
    pub portfolio_items: serde_json::Value,
    pub skills: Vec<SkillDto>,
    pub experiences: Vec<ExperienceDto>,
    pub educations: Vec<EducationDto>,
    pub languages: Vec<LanguageDto>,
    pub references: Vec<ReferenceDto>,
    pub resumes: Vec<ResumeDto>,
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

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SendInvitationCommand {
    #[schema(value_type = String, format = Uuid)]
    pub opportunity_id: Uuid,
    pub message: Option<String>,
}

/// فیلترهای نقشه و دیسکاوری استعدادها برای کارفرما
#[derive(Debug, Clone, Deserialize, Default, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchTalentsRequest {
    pub q: Option<String>,
    pub skill_ids: Option<String>,
    pub city: Option<String>,
    pub actively_looking_only: Option<bool>,
    pub bbox: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_meters: Option<f64>,
    pub limit: Option<usize>,
    pub opportunity_id: Option<Uuid>,
}