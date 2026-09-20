use super::lifecycle::OpportunityStatus;
use crate::error::DomainError;
use crate::taxonomy::ExperienceLevel;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Salary {
    pub min: Option<Decimal>,
    pub max: Option<Decimal>,
    pub currency: String,
    pub period: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub location_ids: Vec<Uuid>,
    pub skill_ids: Vec<Uuid>,
}

impl NewOpportunity {
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.title.trim().is_empty() {
            return Err(DomainError::InvariantViolation("Job title cannot be empty".into()));
        }
        if self.workplace_type == WorkplaceType::Remote && self.remote_scope.is_none() {
            return Err(DomainError::InvariantViolation(
                "Remote opportunities must specify a valid remote scope".into(),
            ));
        }
        if self.workplace_type == WorkplaceType::Onsite && self.location_ids.is_empty() {
            return Err(DomainError::InvariantViolation(
                "Onsite opportunities must have at least one physical location".into(),
            ));
        }
        Ok(())
    }
}