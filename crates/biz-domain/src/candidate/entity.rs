use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub avatar_storage_key: Option<String>,
    pub preferred_location_id: Option<Uuid>,
    pub preferred_city: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateExperience {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub title: String,
    pub company_name: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_current: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateEducation {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub institution: String,
    pub degree: Option<String>,
    pub field_of_study: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateResume {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub storage_key: String,
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
}