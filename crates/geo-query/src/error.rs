use geo_types::GeoError;
use geo_storage::StorageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Geospatial domain error: {0}")]
    Domain(#[from] GeoError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Query limit exceeds maximum allowed threshold of {max}. Requested: {requested}")]
    LimitExceeded { requested: usize, max: usize },

    #[error("Bounding box area is too large for this zoom level")]
    BBoxAreaTooLarge,
}