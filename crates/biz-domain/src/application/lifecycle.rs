use crate::error::DomainError;
use serde::{Deserialize, Serialize};

/// Exact frozen Application lifecycle (Section 20 of MASTER PROMPT 03).
/// SUBMITTED -> REVIEWING -> INTERVIEW -> (ACCEPTED / REJECTED)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Submitted,
    Reviewing,
    Interview,
    Accepted,
    Rejected,
}

impl ApplicationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Reviewing => "reviewing",
            Self::Interview => "interview",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "submitted" => Some(Self::Submitted),
            "reviewing" => Some(Self::Reviewing),
            "interview" => Some(Self::Interview),
            "accepted" => Some(Self::Accepted),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }

    /// Validates allowed state transitions according to section 20
    pub fn can_transition_to(&self, next: Self) -> bool {
        match (self, next) {
            (Self::Submitted, Self::Reviewing) => true,
            (Self::Reviewing, Self::Interview) => true,
            (Self::Reviewing, Self::Rejected) => true,
            (Self::Interview, Self::Accepted) => true,
            (Self::Interview, Self::Rejected) => true,
            _ => false, // ACCEPTED and REJECTED are terminal
        }
    }

    pub fn transition_to(&self, next: Self) -> Result<Self, DomainError> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(DomainError::InvariantViolation(format!(
                "Invalid application state transition from {:?} to {:?}",
                self, next
            )))
        }
    }
}