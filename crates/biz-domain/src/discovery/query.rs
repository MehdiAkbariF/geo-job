use chrono::{DateTime, Utc};
use geo_types::{BoundingBox, GeoPoint, Radius};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Newest,
    Distance,
    SalaryDesc,
    MatchScore,
}

impl Default for SortBy {
    fn default() -> Self {
        Self::Newest
    }
}

impl SortBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Newest => "newest",
            Self::Distance => "distance",
            Self::SalaryDesc => "salary_desc",
            Self::MatchScore => "match_score",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "distance" => Self::Distance,
            "salary_desc" => Self::SalaryDesc,
            "match_score" => Self::MatchScore,
            _ => Self::Newest,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchCursor {
    pub published_at: DateTime<Utc>,
    pub id: Uuid,
}

impl SearchCursor {
    pub fn encode(&self) -> String {
        format!("{}:{}", self.published_at.to_rfc3339(), self.id)
    }

    pub fn decode(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 {
            return None;
        }
        let published_at = DateTime::parse_from_rfc3339(parts[0]).ok()?.with_timezone(&Utc);
        let id = Uuid::parse_str(parts[1]).ok()?;
        Some(Self { published_at, id })
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub category_id: Option<Uuid>,
    pub occupation_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub skill_ids: Vec<Uuid>,
    pub opportunity_type: Option<String>,
    pub workplace_type: Option<String>,
    pub experience_level: Option<String>,
    pub salary_min: Option<rust_decimal::Decimal>,
    pub salary_max: Option<rust_decimal::Decimal>,
    pub include_remote: bool,
    
    pub city: Option<String>,
    pub point: Option<GeoPoint>,
    pub radius: Option<Radius>,
    pub bbox: Option<BoundingBox>,

    pub sort: SortBy,
    pub cursor: Option<SearchCursor>,
    pub limit: usize,
}