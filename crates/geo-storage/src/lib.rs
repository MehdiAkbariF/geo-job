use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_sec: u64,
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