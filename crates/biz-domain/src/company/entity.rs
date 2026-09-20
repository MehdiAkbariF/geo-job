use super::membership::CompanyRole;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyVerificationStatus {
    NotStarted,
    Pending,
    UnderReview,
    Verified,
    Rejected,
    Expired,
}

impl Default for CompanyVerificationStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_storage_key: Option<String>,
    pub website: Option<String>,
    pub verification_status: CompanyVerificationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompanyMembership {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub role: CompanyRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewCompany {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub website: Option<String>,
}