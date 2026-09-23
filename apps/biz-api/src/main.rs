mod error;
mod extractors;
mod handlers;
mod openapi;
mod state;

use axum::{
    routing::{delete, get, post, put},
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

    if let Ok(raw_pass) = biz_domain::identity::RawPassword::new("Password1234!") {
        if let Ok(real_hash) = biz_storage::PasswordService::hash_password(&raw_pass) {
            let _ = sqlx::query("UPDATE users SET password_hash = $1 WHERE email IN ('demo@geojob.ir', 'employer@geojob.ir', 'verified_employer@geojob.ir', 'pending_employer@geojob.ir')")
                .bind(real_hash)
                .execute(&pool)
                .await;
            tracing::info!("Verified cryptographic Argon2id password hash for all demo users");
        }
    }

    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super-secret-jwt-key-change-in-production-123456".to_string());

    let app_state = AppState::new(pool, jwt_secret);

    let api_router = Router::new()
        // Auth
        .route("/auth/register", post(handlers::auth::register_handler))
        .route("/auth/login", post(handlers::auth::login_handler))
        .route("/auth/refresh", post(handlers::auth::refresh_handler))
        .route("/auth/logout", post(handlers::auth::logout_handler))
        .route("/me", get(handlers::auth::me_handler))
        .route("/me/companies", get(handlers::company::list_my_companies_handler))
        
        // Candidate Profile, Experience, Education, Languages, References & Resumes
        .route("/candidates/me", get(handlers::candidate::get_my_profile_handler).put(handlers::candidate::update_my_profile_handler))
        .route("/candidates/me/experiences", post(handlers::candidate::add_experience_handler))
        .route("/candidates/me/experiences/:id", delete(handlers::candidate::delete_experience_handler))
        .route("/candidates/me/educations", post(handlers::candidate::add_education_handler))
        .route("/candidates/me/educations/:id", delete(handlers::candidate::delete_education_handler))
        .route("/candidates/me/languages", post(handlers::candidate::add_language_handler))
        .route("/candidates/me/languages/:id", delete(handlers::candidate::delete_language_handler))
        .route("/candidates/me/references", post(handlers::candidate::add_reference_handler))
        .route("/candidates/me/references/:id", delete(handlers::candidate::delete_reference_handler))
        .route("/candidates/me/resumes", post(handlers::candidate::add_resume_handler))
        .route("/candidates/me/resumes/:id", delete(handlers::candidate::delete_resume_handler))
        .route("/candidates/me/skills", put(handlers::candidate::set_skills_handler))
        .route("/candidates/me/preferences", get(handlers::candidate::get_preferences_handler).put(handlers::candidate::set_preferences_handler))
        .route("/candidates/me/applications", get(handlers::candidate::list_my_applications_handler))

        // Headhunting & Talent Map for Employers
        .route("/talents/search", get(handlers::candidate::search_talents_handler))
        .route("/opportunities/:id/matched-candidates", get(handlers::candidate::get_matched_candidates_handler))
        .route("/candidates/:id/invite", post(handlers::candidate::invite_candidate_handler))

        // Companies Management, Onboarding, Showcase & Branches
        .route("/companies/onboard", post(handlers::company::onboard_company_handler))
        .route("/companies", post(handlers::company::create_company_handler))
        .route("/companies/:id", get(handlers::company::get_company_handler).put(handlers::company::update_company_handler))
        .route("/companies/:id/locations", post(handlers::company::add_company_location_handler).get(handlers::company::list_company_locations_handler))
        .route("/companies/:id/members", get(handlers::company::list_members_handler).post(handlers::company::add_member_handler))
        .route("/companies/:id/opportunities", get(handlers::company::list_company_opportunities_handler).post(handlers::opportunity::create_opportunity_handler))
        .route("/companies/:id/public-opportunities", get(handlers::company::list_public_company_opportunities_handler))

        // Discovery / Search, Recommended Feed & Map Bridge
        .route("/opportunities/search", get(handlers::discovery::search_opportunities_handler))
        .route("/opportunities/recommended", get(handlers::discovery::get_recommended_opportunities_handler))
        .route("/opportunities/map-pins", get(handlers::discovery::get_map_pins_handler))
        .route("/opportunities/by-location/:location_id", get(handlers::discovery::get_opportunities_by_location_handler))
        .route("/opportunities/:id", get(handlers::opportunity::get_opportunity_handler))
        .route("/opportunities/:id/track-click", post(handlers::opportunity::track_opportunity_click_handler))

        // Opportunities Lifecycle & Paid Map Promotions (بدون هیچ روت تکراری)
        .route("/opportunities/:id/publish", post(handlers::opportunity::publish_opportunity_handler))
        .route("/opportunities/:id/pause", post(handlers::opportunity::pause_opportunity_handler))
        .route("/opportunities/:id/resume", post(handlers::opportunity::resume_opportunity_handler))
        .route("/opportunities/:id/close", post(handlers::opportunity::close_opportunity_handler))
        .route("/opportunities/:id/ladder", post(handlers::opportunity::ladder_opportunity_handler))
        .route("/opportunities/:id/feature-pin", post(handlers::opportunity::feature_opportunity_pin_handler))

        // Employer ATS / Applications
        .route("/opportunities/:id/applications", post(handlers::application::submit_application_handler).get(handlers::company::list_opportunity_applicants_handler))
        .route("/applications/:id", get(handlers::application::get_application_dossier_handler))
        .route("/applications/:id/status", post(handlers::application::change_application_status_handler))

        // Saved Items (Opportunities, Companies, Searches)
        .route("/opportunities/:id/save", post(handlers::saved::save_opportunity_handler).delete(handlers::saved::remove_saved_opportunity_handler))
        .route("/me/saved-opportunities", get(handlers::saved::list_saved_opportunities_handler))
        .route("/companies/:id/save", post(handlers::saved::save_company_handler).delete(handlers::saved::remove_saved_company_handler))
        .route("/me/saved-companies", get(handlers::saved::list_saved_companies_handler))
        .route("/me/saved-searches", post(handlers::saved::save_search_handler).get(handlers::saved::list_saved_searches_handler))

        // 💳 زیرساخت مالی، کیف پول، تعرفه‌ها، فاکتورها و خرید دسترسی کارجویان
        .route("/companies/:id/wallet", get(handlers::finance::get_company_wallet_handler))
        .route("/finance/tariffs", get(handlers::finance::list_tariffs_handler))
        .route("/companies/:id/finance/invoices", post(handlers::finance::create_invoice_handler))
        .route("/companies/:id/finance/invoices/:invoice_id/pay-mock", post(handlers::finance::mock_pay_invoice_handler))
        .route("/companies/:id/finance/transactions", get(handlers::finance::list_wallet_transactions_handler))
        .route("/companies/:id/talents/:candidate_id/unlock", post(handlers::finance::unlock_talent_contact_handler))

        // Taxonomies
        .route("/taxonomies/categories", get(handlers::taxonomy::list_categories_handler))
        .route("/taxonomies/industries", get(handlers::taxonomy::list_industries_handler))
        .route("/taxonomies/occupations", get(handlers::taxonomy::list_occupations_handler))
        .route("/taxonomies/skills/resolve", get(handlers::taxonomy::resolve_skill_handler))
        .route("/taxonomies/countries", get(handlers::taxonomy::list_countries_handler))
        .route("/taxonomies/cities", get(handlers::taxonomy::list_cities_handler))

        // Governance, Verification & Reports
        .route("/opportunities/:id/reports", post(handlers::governance::report_opportunity_handler))
        .route("/companies/:id/verifications", post(handlers::governance::submit_verification_handler))
        .route("/admin/verifications", get(handlers::governance::list_pending_verifications_handler))
        .route("/admin/verifications/:id/review", post(handlers::governance::review_verification_handler))
        .route("/admin/reports", get(handlers::governance::list_reports_handler));

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