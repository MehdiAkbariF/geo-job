use crate::error::DomainError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Production ATS Pipeline Lifecycle
/// SUBMITTED -> REVIEWING -> INTERVIEW -> ACCEPTED (or REJECTED from any stage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
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
        match s.trim().to_lowercase().as_str() {
            "submitted" => Some(Self::Submitted),
            "reviewing" => Some(Self::Reviewing),
            "interview" => Some(Self::Interview),
            "accepted" => Some(Self::Accepted),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }

    /// Flexible & real-world ATS state transition validation
    pub fn can_transition_to(&self, next: Self) -> bool {
        if *self == next {
            return true;
        }

        match (self, next) {
            // ۱. از هر مرحله‌ای امکان رد درخواست وجود دارد
            (_, Self::Rejected) => true,

            // ۲. پیشروی از مرحله ثبت اولیه
            (Self::Submitted, Self::Reviewing) => true,
            (Self::Submitted, Self::Interview) => true,
            (Self::Submitted, Self::Accepted) => true,

            // ۳. پیشروی از مرحله بررسی اولیه
            (Self::Reviewing, Self::Interview) => true,
            (Self::Reviewing, Self::Accepted) => true,
            (Self::Reviewing, Self::Submitted) => true,

            // ۴. پیشروی از مرحله مصاحبه
            (Self::Interview, Self::Accepted) => true,
            (Self::Interview, Self::Reviewing) => true,

            // ۵. امکان بازگشایی و تغییر وضعیت پس از قبولی یا رد
            (Self::Accepted, Self::Interview) => true,
            (Self::Accepted, Self::Reviewing) => true,
            (Self::Rejected, Self::Reviewing) => true,
            (Self::Rejected, Self::Interview) => true,

            // سایر حالات غیرمجاز
            _ => false,
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