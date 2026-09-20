use crate::dto::CreateLocationRequest;
use crate::error::ApiError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use geo_domain::{LocationPrecision, NewLocation};
use geo_types::GeoPoint;

pub async fn create_location(
    State(state): State<AppState>,
    Json(payload): Json<CreateLocationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let point = GeoPoint::new(payload.longitude, payload.latitude)
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let new_loc = NewLocation {
        point,
        address_summary: payload.address_summary,
        precision: payload.precision.unwrap_or(LocationPrecision::Exact),
        source: payload.source.unwrap_or_else(|| "api".to_string()),
        source_id: payload.source_id,
        metadata: payload.metadata.unwrap_or_else(|| serde_json::json!({})),
    };

    let created = state.location_repo.insert(&new_loc).await?;
    Ok((StatusCode::CREATED, Json(created)))
}