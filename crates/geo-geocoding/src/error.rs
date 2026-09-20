use geo_types::GeoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeocodingError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Geospatial domain error: {0}")]
    Domain(#[from] GeoError),

    #[error("Area with id {0} not found")]
    NotFound(uuid::Uuid),
}