use geo_domain::Location;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lightweight presentation model specifically designed for fast client-side marker rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkerDto {
    pub id: Uuid,
    pub coordinates: [f64; 2],
    pub label: Option<String>,
}

impl From<Location> for MarkerDto {
    fn from(loc: Location) -> Self {
        Self {
            id: loc.id,
            coordinates: loc.point.to_coordinates(),
            label: loc.address_summary,
        }
    }
}