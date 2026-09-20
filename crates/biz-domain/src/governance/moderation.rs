use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Platform administration roles (Section 46, 62 & 109).
/// Distinct from CompanyMembership roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminRole {
    Admin,
    Moderator,
    Support,
}

impl AdminRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Moderator => "moderator",
            Self::Support => "support",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Self::Admin),
            "moderator" => Some(Self::Moderator),
            "support" => Some(Self::Support),
            _ => None,
        }
    }

    pub fn can_moderate(&self) -> bool {
        matches!(self, Self::Admin | Self::Moderator)
    }

    pub fn can_verify_companies(&self) -> bool {
        matches!(self, Self::Admin | Self::Moderator)
    }
}

/// Report is an allegation, not a decision (Section 44 & 58)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub reporter_user_id: Option<Uuid>,
    pub opportunity_id: Uuid,
    pub reason: String,
    pub details: Option<String>,
}

/// ModerationCase represents the actual review process (Section 44 & 59)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationCase {
    pub id: Uuid,
    pub opportunity_id: Uuid,
    pub status: String,
    pub assigned_moderator_id: Option<Uuid>,
    pub action_taken: String,
    pub internal_notes: Option<String>, // Staff-only private notes
}