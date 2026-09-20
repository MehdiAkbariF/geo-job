use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use geo_geocoding::{find_areas_by_parent, reverse_geocode_point};
use geo_types::GeoPoint;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ReverseGeocodeQuery {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Deserialize)]
pub struct AdminAreasQuery {
    pub parent_id: Option<Uuid>,
}

pub async fn reverse_geocode_handler(
    State(state): State<AppState>,
    Query(params): Query<ReverseGeocodeQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let point = GeoPoint::new(params.lon, params.lat)
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    match reverse_geocode_point(&state.pool, &point).await {
        Ok(Some(result)) => Ok(Json(result)),
        Ok(None) => Err(ApiError::NotFound(
            "No administrative area found containing these coordinates".to_string(),
        )),
        Err(e) => Err(ApiError::Validation(e.to_string())),
    }
}

pub async fn list_admin_areas_handler(
    State(state): State<AppState>,
    Query(params): Query<AdminAreasQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let areas = find_areas_by_parent(&state.pool, params.parent_id)
        .await
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    Ok(Json(areas))
}