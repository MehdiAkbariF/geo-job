use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyRole {
    Owner,
    Admin,
    Recruiter,
    HiringManager,
}

impl CompanyRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Recruiter => "recruiter",
            Self::HiringManager => "hiring_manager",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(Self::Owner),
            "admin" => Some(Self::Admin),
            "recruiter" => Some(Self::Recruiter),
            "hiring_manager" => Some(Self::HiringManager),
            _ => None,
        }
    }

    pub fn can_manage_members(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_edit_company(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    pub fn can_manage_opportunities(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Recruiter)
    }

    pub fn can_review_applications(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Recruiter | Self::HiringManager)
    }
}