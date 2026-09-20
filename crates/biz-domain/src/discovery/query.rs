use geo_types::{BoundingBox, GeoPoint, Radius};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Newest,
    Distance,
    SalaryDesc,
}

impl Default for SortBy {
    fn default() -> Self {
        Self::Newest
    }
}

/// Runtime Search & Discovery Query value object (Section 31 & 32 of MASTER PROMPT 03).
/// MUST NOT be persisted as a business entity.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: Option<String>,
    pub category_id: Option<Uuid>,
    pub occupation_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub opportunity_type: Option<String>,
    pub workplace_type: Option<String>,
    pub experience_level: Option<String>,
    pub salary_min: Option<rust_decimal::Decimal>,
    
    // Spatial constraints (orchestrated with PostGIS)
    pub point: Option<GeoPoint>,
    pub radius: Option<Radius>,
    pub bbox: Option<BoundingBox>,

    // Pagination & Sorting
    pub sort: SortBy,
    pub cursor: Option<String>,
    pub limit: usize,
}