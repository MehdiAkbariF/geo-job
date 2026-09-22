use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Candidate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub avatar_storage_key: Option<String>,
    pub residence_location_id: Option<Uuid>,
    pub preferred_city: Option<String>,
    pub preferred_commute_center_id: Option<Uuid>,
    pub preferred_commute_radius_meters: Option<i32>,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CandidateExperience {
    pub id: Uuid,
    pub candidate_id: Uuid,
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
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CandidateEducation {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub institution: String,
    pub degree_level: String,
    pub field_of_study: Option<String>,
    pub gpa: Option<Decimal>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CandidateLanguage {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub language_name: String,
    pub proficiency_level: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CandidateReference {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub full_name: String,
    pub organization_name: String,
    pub job_title: String,
    pub relationship_type: Option<String>,
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    pub is_still_colleagues: bool,
    pub phone: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CandidateResume {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub storage_key: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
}