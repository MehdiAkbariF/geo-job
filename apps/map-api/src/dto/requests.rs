use crate::error::ApiError;
use geo_domain::LocationPrecision;
use geo_types::{BoundingBox, GeoPoint, Radius};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BBoxQueryParams {
    pub bbox: String,
    pub zoom: Option<u8>,
    pub limit: Option<usize>,
    pub source: Option<String>,
}

impl BBoxQueryParams {
    pub fn parse_bbox(&self) -> Result<BoundingBox, ApiError> {
        let parts: Vec<&str> = self.bbox.split(',').map(|s| s.trim()).collect();
        if parts.len() != 4 {
            return Err(ApiError::Validation(
                "Bounding box must be formatted as 'west,south,east,north'".to_string(),
            ));
        }

        let west: f64 = parts[0].parse().map_err(|_| ApiError::Validation("Invalid west coordinate".into()))?;
        let south: f64 = parts[1].parse().map_err(|_| ApiError::Validation("Invalid south coordinate".into()))?;
        let east: f64 = parts[2].parse().map_err(|_| ApiError::Validation("Invalid east coordinate".into()))?;
        let north: f64 = parts[3].parse().map_err(|_| ApiError::Validation("Invalid north coordinate".into()))?;

        BoundingBox::new(west, south, east, north)
            .map_err(|e| ApiError::Validation(e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
pub struct NearbyQueryParams {
    pub lat: f64,
    pub lon: f64,
    pub radius: f64,
    pub limit: Option<usize>,
}

impl NearbyQueryParams {
    pub fn parse_point_and_radius(&self) -> Result<(GeoPoint, Radius), ApiError> {
        let pt = GeoPoint::new(self.lon, self.lat)
            .map_err(|e| ApiError::Validation(e.to_string()))?;
        let rad = Radius::from_meters(self.radius)
            .map_err(|e| ApiError::Validation(e.to_string()))?;
        Ok((pt, rad))
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateLocationRequest {
    pub longitude: f64,
    pub latitude: f64,
    pub address_summary: Option<String>,
    pub precision: Option<LocationPrecision>,
    pub source: Option<String>,
    pub source_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}