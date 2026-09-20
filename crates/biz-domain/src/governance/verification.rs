use crate::error::DomainError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Exact frozen Verification lifecycle (Section 55 of MASTER PROMPT 03).
/// NOT_STARTED -> PENDING -> UNDER_REVIEW -> (VERIFIED / REJECTED) -> EXPIRED
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    NotStarted,
    Pending,
    UnderReview,
    Verified,
    Rejected,
    Expired,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotStarted => "not_started",
            Self::Pending => "pending",
            Self::UnderReview => "under_review",
            Self::Verified => "verified",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "not_started" => Some(Self::NotStarted),
            "pending" => Some(Self::Pending),
            "under_review" => Some(Self::UnderReview),
            "verified" => Some(Self::Verified),
            "rejected" => Some(Self::Rejected),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }

    pub fn can_transition_to(&self, next: Self) -> bool {
        match (self, next) {
            (Self::NotStarted, Self::Pending) => true,
            (Self::Pending, Self::UnderReview) => true,
            (Self::UnderReview, Self::Verified) => true,
            (Self::UnderReview, Self::Rejected) => true,
            (Self::Verified, Self::Expired) => true,
            _ => false,
        }
    }

    pub fn transition_to(&self, next: Self) -> Result<Self, DomainError> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(DomainError::InvariantViolation(format!(
                "Invalid verification transition from {:?} to {:?}",
                self, next
            )))
        }
    }
}