use chrono::{DateTime, Utc};
use geo_domain::{Location, LocationPrecision};
use geo_types::GeoPoint;
use crate::error::StorageError;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct LocationDbRow {
    pub id: Uuid,
    pub longitude: f64,
    pub latitude: f64,
    pub address_summary: Option<String>,
    pub precision: String,
    pub source: String,
    pub source_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<LocationDbRow> for Location {
    type Error = StorageError;

    fn try_from(row: LocationDbRow) -> Result<Self, Self::Error> {
        let point = GeoPoint::new(row.longitude, row.latitude)?;
        let precision = match row.precision.as_str() {
            "exact" => LocationPrecision::Exact,
            "rooftop" => LocationPrecision::Rooftop,
            "street" => LocationPrecision::Street,
            "neighborhood" => LocationPrecision::Neighborhood,
            "city" => LocationPrecision::City,
            _ => LocationPrecision::Approximate,
        };

        Ok(Self {
            id: row.id,
            point,
            address_summary: row.address_summary,
            precision,
            source: row.source,
            source_id: row.source_id,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}