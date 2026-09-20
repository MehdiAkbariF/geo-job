use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use geo_geocoding::{find_areas_by_parent, reverse_geocode_point, AdministrativeArea, ReverseGeocodeResult};
use geo_types::GeoPoint;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ReverseGeocodeQuery {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AdminAreasQuery {
    pub parent_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/api/v1/reverse-geocoding",
    params(ReverseGeocodeQuery),
    responses(
        (status = 200, description = "Address resolved", body = ReverseGeocodeResult),
        (status = 404, description = "Area not found")
    ),
    tag = "Geocoding"
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/admin/areas",
    params(AdminAreasQuery),
    responses(
        (status = 200, description = "List administrative areas", body = Vec<AdministrativeArea>)
    ),
    tag = "Geocoding"
)]
pub async fn list_admin_areas_handler(
    State(state): State<AppState>,
    Query(params): Query<AdminAreasQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let areas = find_areas_by_parent(&state.pool, params.parent_id)
        .await
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    Ok(Json(areas))
}