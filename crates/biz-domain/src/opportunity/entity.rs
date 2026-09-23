use super::lifecycle::OpportunityStatus;
use crate::error::DomainError;
use crate::taxonomy::ExperienceLevel;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityType {
    FullTime,
    PartTime,
    Contract,
    Internship,
    Freelance,
    Temporary,
}

impl OpportunityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FullTime => "full_time",
            Self::PartTime => "part_time",
            Self::Contract => "contract",
            Self::Internship => "internship",
            Self::Freelance => "freelance",
            Self::Temporary => "temporary",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "full_time" => Some(Self::FullTime),
            "part_time" => Some(Self::PartTime),
            "contract" => Some(Self::Contract),
            "internship" => Some(Self::Internship),
            "freelance" => Some(Self::Freelance),
            "temporary" => Some(Self::Temporary),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkplaceType {
    Onsite,
    Hybrid,
    Remote,
}

impl WorkplaceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Onsite => "onsite",
            Self::Hybrid => "hybrid",
            Self::Remote => "remote",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "onsite" => Some(Self::Onsite),
            "hybrid" => Some(Self::Hybrid),
            "remote" => Some(Self::Remote),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RemoteScope {
    Global,
    Country,
    Region,
    Timezone,
}

impl RemoteScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Country => "country",
            Self::Region => "region",
            Self::Timezone => "timezone",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "global" => Some(Self::Global),
            "country" => Some(Self::Country),
            "region" => Some(Self::Region),
            "timezone" => Some(Self::Timezone),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Salary {
    pub min: Option<Decimal>,
    pub max: Option<Decimal>,
    pub currency: String,
    pub period: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Opportunity {
    pub id: Uuid,
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    pub category_id: Uuid,
    pub occupation_id: Option<Uuid>,
    pub opportunity_type: OpportunityType,
    pub workplace_type: WorkplaceType,
    pub remote_scope: Option<RemoteScope>,
    pub experience_level: ExperienceLevel,
    pub salary: Salary,
    pub status: OpportunityStatus,
    pub is_urgent: bool,
    pub is_featured: bool,
    pub working_hours: Option<String>,
    pub gender_preference: String,
    pub has_insurance: bool,
    pub laddered_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewOpportunity {
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    pub category_id: Uuid,
    pub occupation_id: Option<Uuid>,
    pub opportunity_type: OpportunityType,
    pub workplace_type: WorkplaceType,
    pub remote_scope: Option<RemoteScope>,
    pub experience_level: ExperienceLevel,
    pub salary: Salary,
    pub is_urgent: bool,
    pub working_hours: Option<String>,
    pub gender_preference: String,
    pub has_insurance: bool,
    pub location_ids: Vec<Uuid>,
    pub skill_ids: Vec<Uuid>,
}

impl NewOpportunity {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.title.trim().is_empty() {
            return Err(DomainError::InvariantViolation("عنوان آگهی نمی‌تواند خالی باشد".into()));
        }
        if self.description.trim().is_empty() {
            return Err(DomainError::InvariantViolation("شرح آگهی نمی‌تواند خالی باشد".into()));
        }
        if self.workplace_type != WorkplaceType::Remote && self.location_ids.is_empty() {
            return Err(DomainError::InvariantViolation("برای آگهی‌های حضوری یا هیبرید، حداقل یک شعبه باید انتخاب شود".into()));
        }
        Ok(())
    }
}