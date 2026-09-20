mod dto;
mod error;
mod handlers;
mod openapi;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use geo_storage::{create_connection_pool, run_migrations, DatabaseConfig};
use openapi::MapApiDoc;
use state::AppState;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "map_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initializing Geospatial Map API Service...");

    let db_config = DatabaseConfig {
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://map_user:map_password@localhost:5433/map_platform".to_string()
        }),
        max_connections: 20,
        min_connections: 5,
        connect_timeout_sec: 10,
    };

    let pool = create_connection_pool(&db_config).await?;
    tracing::info!("Connected to PostgreSQL/PostGIS. Running migrations...");
    run_migrations(&pool).await?;
    tracing::info!("Migrations applied successfully.");

    let app_state = AppState::new(pool);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", MapApiDoc::openapi()))
        .route("/healthz", get(handlers::health::health_check))
        .route("/api/v1/map/features", get(handlers::map::search_viewport))
        .route("/api/v1/map/nearby", get(handlers::map::search_nearby))
        .route("/api/v1/locations", post(handlers::locations::create_location))
        .route("/api/v1/tiles/:z/:x/:tile", get(handlers::tiles::serve_vector_tile))
        .route("/api/v1/base-tiles/:z/:x/:tile", get(handlers::tiles::serve_base_map_tile))
        .route("/api/v1/reverse-geocoding", get(handlers::geocoding::reverse_geocode_handler))
        .route("/api/v1/admin/areas", get(handlers::geocoding::list_admin_areas_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Map Platform HTTP server listening on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}