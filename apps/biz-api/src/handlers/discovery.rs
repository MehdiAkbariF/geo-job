use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use biz_application::discovery::SearchOpportunitiesRequest;
use biz_domain::discovery::{OpportunitySearchResult, SearchPageResult};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/v1/opportunities/search",
    responses((status = 200, description = "Search results returned successfully", body = SearchPageResult)),
    tag = "Discovery"
)]
pub async fn search_opportunities_handler(
    State(state): State<AppState>,
    Query(req): Query<SearchOpportunitiesRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.discovery_use_cases.search(req).await?;
    Ok(Json(result))
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