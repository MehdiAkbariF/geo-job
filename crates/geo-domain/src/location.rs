use chrono::{DateTime, Utc};
use geo_types::GeoPoint;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LocationPrecision {
    Exact,
    Rooftop,
    Street,
    Neighborhood,
    City,
    Approximate,
}

impl Default for LocationPrecision {
    fn default() -> Self {
        Self::Exact
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Location {
    pub id: Uuid,
    #[schema(value_type = [f64; 2], example = json!([51.3890, 35.6892]))]
    pub point: GeoPoint,
    pub address_summary: Option<String>,
    pub precision: LocationPrecision,
    pub source: String,
    pub source_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewLocation {
    pub point: GeoPoint,
    pub address_summary: Option<String>,
    pub precision: LocationPrecision,
    pub source: String,
    pub source_id: Option<String>,
    pub metadata: serde_json::Value,
}