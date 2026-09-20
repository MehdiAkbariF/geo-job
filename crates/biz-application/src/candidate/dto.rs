use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileCommand {
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub preferred_city: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddExperienceCommand {
    pub title: String,
    pub company_name: String,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub is_current: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateProfileDto {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub headline: Option<String>,
    pub bio: Option<String>,
    pub preferred_city: Option<String>,
}