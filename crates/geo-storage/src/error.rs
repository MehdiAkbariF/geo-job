use geo_types::GeoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Geospatial domain error: {0}")]
    Domain(#[from] GeoError),

    #[error("Location with id {0} not found")]
    NotFound(uuid::Uuid),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}