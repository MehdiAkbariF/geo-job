use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExperienceLevel {
    Entry,
    Junior,
    MidLevel,
    Senior,
    Lead,
    Executive,
}

impl ExperienceLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Entry => "entry",
            Self::Junior => "junior",
            Self::MidLevel => "mid_level",
            Self::Senior => "senior",
            Self::Lead => "lead",
            Self::Executive => "executive",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "entry" => Some(Self::Entry),
            "junior" => Some(Self::Junior),
            "mid_level" => Some(Self::MidLevel),
            "senior" => Some(Self::Senior),
            "lead" => Some(Self::Lead),
            "executive" => Some(Self::Executive),
            _ => None,
        }
    }
}