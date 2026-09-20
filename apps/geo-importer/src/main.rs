mod cli;
mod error;
mod osm_parser;
mod pipeline;

use clap::Parser;
use cli::CliArgs;
use geo_storage::{create_connection_pool, run_migrations, DatabaseConfig};
use pipeline::{insert_osm_batch, seed_sample_locations};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = CliArgs::parse();

    let db_url = args
        .database_url
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .unwrap_or_else(|| {
            "postgres://map_user:map_password@localhost:5433/map_platform".to_string()
        });

    let db_config = DatabaseConfig {
        database_url: db_url,
        max_connections: 10,
        min_connections: 2,
        connect_timeout_sec: 10,
    };

    tracing::info!("Connecting to PostGIS database...");
    let pool = create_connection_pool(&db_config).await?;
    run_migrations(&pool).await?;

    if args.seed_samples {
        tracing::info!("Seeding initial sample locations...");
        seed_sample_locations(&pool).await?;
        tracing::info!("Seeding completed successfully.");
    }

    if let Some(pbf_path) = args.file {
        tracing::info!("Starting streaming import from OSM PBF: {:?}", pbf_path);
        let rt = tokio::runtime::Handle::current();

        let total = osm_parser::stream_osm_pbf(
            &pbf_path,
            |batch| {
                let pool_ref = pool.clone();
                rt.block_on(async move {
                    insert_osm_batch(&pool_ref, &batch).await
                })?;
                Ok(())
            },
            args.batch_size,
        )?;

        tracing::info!("Streaming import finished! Total locations ingested: {}", total);
    }

    Ok(())
}