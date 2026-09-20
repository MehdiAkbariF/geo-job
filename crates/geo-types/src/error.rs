use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum GeoError {
    #[error("Invalid latitude: {0}. Latitude must be between -90.0 and +90.0 degrees.")]
    InvalidLatitude(f64),

    #[error("Invalid longitude: {0}. Longitude must be between -180.0 and +180.0 degrees.")]
    InvalidLongitude(f64),

    #[error("Invalid BoundingBox coordinates: south ({south}) must be <= north ({north}).")]
    InvalidBoundingBoxLatitude { south: f64, north: f64 },

    #[error("Invalid BoundingBox coordinates: west ({west}) must be <= east ({east}).")]
    InvalidBoundingBoxLongitude { west: f64, east: f64 },

    #[error("Invalid radius: {0} meters. Radius must be greater than zero.")]
    InvalidRadius(f64),
}