pub mod error;
pub mod models;
pub mod repository;

pub use error::StorageError;
pub use repository::LocationRepository;

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_sec: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://map_user:map_password@localhost:5432/map_platform".to_string(),
            max_connections: 20,
            min_connections: 2,
            connect_timeout_sec: 10,
        }
    }
}

pub async fn create_connection_pool(
    config: &DatabaseConfig,
) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_sec))
        .connect(&config.database_url)
        .await
}

/// Applies all pending SQL migrations to the database.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(pool).await
}