use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use biz_application::discovery::SearchOpportunitiesRequest;

pub async fn search_opportunities_handler(
    State(state): State<AppState>,
    Query(req): Query<SearchOpportunitiesRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state.discovery_use_cases.search(req).await?;
    Ok(Json(result))
}