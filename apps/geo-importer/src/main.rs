mod cli;
mod error;
mod osm_parser;
mod pipeline;

use clap::Parser;
use cli::CliArgs;
use error::ImporterError;
use geo_storage::{create_connection_pool, run_migrations, DatabaseConfig};
use osm_parser::{ExtractedOsmPoint, ExtractedOsmRoad};
use pipeline::{insert_osm_batch, insert_roads_batch};
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

    if let Some(pbf_path) = args.file {
        tracing::info!("Processing real OSM data from PBF: {:?}", pbf_path);

        let (points_tx, mut points_rx) = tokio::sync::mpsc::channel::<Vec<ExtractedOsmPoint>>(16);
        let (roads_tx, mut roads_rx) = tokio::sync::mpsc::channel::<Vec<ExtractedOsmRoad>>(16);
        let batch_size = args.batch_size;

        // Producer Thread: Streaming parse of actual nodes and ways
        let producer_handle = tokio::task::spawn_blocking(move || {
            osm_parser::stream_osm_pbf_full(
                &pbf_path,
                |batch| {
                    if points_tx.blocking_send(batch).is_err() {
                        return Err(ImporterError::Validation("Points channel closed".into()));
                    }
                    Ok(())
                },
                |batch| {
                    if roads_tx.blocking_send(batch).is_err() {
                        return Err(ImporterError::Validation("Roads channel closed".into()));
                    }
                    Ok(())
                },
                batch_size,
            )
        });

        // Consumers: Concurrently write real points and real road centerlines
        let pool_for_points = pool.clone();
        let points_consumer = tokio::spawn(async move {
            let mut count = 0;
            while let Some(batch) = points_rx.recv().await {
                let len = batch.len();
                let _ = insert_osm_batch(&pool_for_points, &batch).await;
                count += len;
                if count % 20000 == 0 {
                    tracing::info!("Imported {} real place/amenity locations...", count);
                }
            }
            count
        });

        let pool_for_roads = pool.clone();
        let roads_consumer = tokio::spawn(async move {
            let mut count = 0;
            while let Some(batch) = roads_rx.recv().await {
                let len = batch.len();
                let _ = insert_roads_batch(&pool_for_roads, &batch).await;
                count += len;
                if count % 10000 == 0 {
                    tracing::info!("Imported {} real road linestrings...", count);
                }
            }
            count
        });

        let _ = producer_handle.await??;
        let total_points = points_consumer.await?;
        let total_roads = roads_consumer.await?;

        tracing::info!(
            "REAL DATA IMPORT COMPLETED! Successfully ingested {} real locations and {} real road centerlines directly into PostGIS.",
            total_points,
            total_roads
        );
    }

    Ok(())
}