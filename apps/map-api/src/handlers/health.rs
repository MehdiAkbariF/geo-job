use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Map service is healthy"),
        (status = 503, description = "Database connection error")
    ),
    tag = "Health"
)]
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "status": "healthy", "database": "connected" })),
        ),
        Err(err) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "unhealthy", "database_error": err.to_string() })),
        ),
    }
}