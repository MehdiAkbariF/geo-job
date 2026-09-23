use super::membership::CompanyRole;
use crate::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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

pub type VerificationStatus = CompanyVerificationStatus;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_storage_key: Option<String>,
    pub website: Option<String>,
    pub business_type: String, // corporate, retail_shop, restaurant_cafe, clinic_office, workshop
    pub trade_license_number: Option<String>,
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
    pub business_type: String,
    pub trade_license_number: Option<String>,
}

impl NewCompany {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.name.trim().is_empty() {
            return Err(DomainError::InvariantViolation("نام کسب‌وکار یا سازمان نمی‌تواند خالی باشد".into()));
        }
        if self.slug.trim().is_empty() {
            return Err(DomainError::InvariantViolation("شناسه انگلیسی (Slug) الزامی است".into()));
        }
        Ok(())
    }
}