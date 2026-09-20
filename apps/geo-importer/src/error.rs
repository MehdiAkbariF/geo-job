use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImporterError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OSM PBF parsing error: {0}")]
    Osm(#[from] osmpbf::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),
}