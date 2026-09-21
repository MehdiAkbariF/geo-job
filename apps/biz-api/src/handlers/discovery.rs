use crate::error::ApiError;
use crate::extractors::auth::MaybeAuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use biz_application::discovery::{GetMapPinsRequest, SearchOpportunitiesRequest};
use biz_domain::discovery::{MapPinSummary, OpportunitySearchResult, SearchPageResult};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/v1/opportunities/search",
    params(SearchOpportunitiesRequest),
    responses((status = 200, description = "Search results returned successfully", body = SearchPageResult)),
    tag = "Discovery"
)]
pub async fn search_opportunities_handler(
    State(state): State<AppState>,
    maybe_auth: MaybeAuthenticatedUser,
    Query(req): Query<SearchOpportunitiesRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.discovery_use_cases.search(maybe_auth.0, req).await?;
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/v1/opportunities/map-pins",
    params(GetMapPinsRequest),
    responses((status = 200, description = "Aggregated map pin clusters with job counts", body = Vec<MapPinSummary>)),
    tag = "Discovery"
)]
pub async fn get_map_pins_handler(
    State(state): State<AppState>,
    Query(req): Query<GetMapPinsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let pins = state.discovery_use_cases.get_map_pins(req).await?;
    Ok(Json(pins))
}

#[utoipa::path(
    get,
    path = "/api/v1/opportunities/by-location/{location_id}",
    responses((status = 200, description = "Active opportunities at this specific map marker", body = Vec<OpportunitySearchResult>)),
    tag = "Discovery"
)]
pub async fn get_opportunities_by_location_handler(
    State(state): State<AppState>,
    Path(location_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let items = state.discovery_use_cases.get_by_location(location_id).await?;
    Ok(Json(items))
}