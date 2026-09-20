mod error;
mod extractors;
mod handlers;
mod openapi;
mod state;

use axum::{
    routing::{get, post},
    Router,
};
use geo_storage::{create_connection_pool, run_migrations, DatabaseConfig};
use openapi::ApiDoc;
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
                .unwrap_or_else(|_| "biz_api=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Initializing Map-First Business Backend API Service...");

    let db_config = DatabaseConfig {
        database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://map_user:map_password@localhost:5433/map_platform".to_string()
        }),
        max_connections: 20,
        min_connections: 5,
        connect_timeout_sec: 10,
    };

    let pool = create_connection_pool(&db_config).await?;
    tracing::info!("Connected to PostgreSQL/PostGIS. Running business migrations...");
    run_migrations(&pool).await?;
    tracing::info!("Business migrations applied successfully.");

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-jwt-key-change-in-production-123456".to_string());

    let app_state = AppState::new(pool, jwt_secret);

    // REST API Routes (/api/v1)
    let api_router = Router::new()
        // Auth
        .route("/auth/register", post(handlers::auth::register_handler))
        .route("/auth/login", post(handlers::auth::login_handler))
        .route("/me", get(handlers::auth::me_handler))
        // Discovery / Search
        .route("/opportunities/search", get(handlers::discovery::search_opportunities_handler))
        // Opportunities Lifecycle
        .route("/companies/:company_id/opportunities", post(handlers::opportunity::create_opportunity_handler))
        .route("/opportunities/:id/publish", post(handlers::opportunity::publish_opportunity_handler))
        .route("/opportunities/:id/pause", post(handlers::opportunity::pause_opportunity_handler))
        .route("/opportunities/:id/close", post(handlers::opportunity::close_opportunity_handler))
        // Applications
        .route("/opportunities/:id/applications", post(handlers::application::submit_application_handler))
        .route("/applications/:id/status", post(handlers::application::change_application_status_handler))
        // Saved Items
        .route("/opportunities/:id/save", post(handlers::saved::save_opportunity_handler).delete(handlers::saved::remove_saved_opportunity_handler))
        .route("/me/saved-opportunities", get(handlers::saved::list_saved_opportunities_handler))
        // Governance / Reports
        .route("/opportunities/:id/reports", post(handlers::governance::report_opportunity_handler));

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1", api_router)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    tracing::info!("Business API Server listening on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}