use crate::error::DomainError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStatus {
    Draft,
    Published,
    Paused,
    Closed,
}

impl OpportunityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Paused => "paused",
            Self::Closed => "closed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "draft" => Some(Self::Draft),
            "published" => Some(Self::Published),
            "paused" => Some(Self::Paused),
            "closed" => Some(Self::Closed),
            _ => None,
        }
    }

    pub fn can_transition_to(&self, next: Self) -> bool {
        match (self, next) {
            (Self::Draft, Self::Published) => true,
            (Self::Published, Self::Paused) => true,
            (Self::Paused, Self::Published) => true,
            (Self::Published, Self::Closed) => true,
            (Self::Paused, Self::Closed) => true,
            _ => false,
        }
    }

    pub fn transition_to(&self, next: Self) -> Result<Self, DomainError> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(DomainError::InvariantViolation(format!(
                "Invalid state transition from {:?} to {:?}",
                self, next
            )))
        }
    }
}