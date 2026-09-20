use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedOpportunity {
    pub candidate_id: Uuid,
    pub opportunity_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedCompany {
    pub candidate_id: Uuid,
    pub company_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub title: String,
    pub criteria: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidatePreferences {
    pub candidate_id: Uuid,
    pub preferred_workplace_types: Vec<String>,
    pub preferred_opportunity_types: Vec<String>,
    pub expected_salary_min: Option<Decimal>,
    pub salary_currency: String,
    pub remote_only: bool,
    pub updated_at: DateTime<Utc>,
}