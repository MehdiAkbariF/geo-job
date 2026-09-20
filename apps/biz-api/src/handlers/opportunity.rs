use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::opportunity::CreateOpportunityCommand;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/companies/{company_id}/opportunities",
    responses((status = 201, description = "Opportunity created")),
    tag = "Opportunities"
)]
pub async fn create_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    Json(mut cmd): Json<CreateOpportunityCommand>,
) -> Result<impl IntoResponse, ApiError> {
    cmd.company_id = company_id;
    let opp = state.opp_use_cases.create_opportunity(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(opp)))
}

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/publish",
    responses((status = 200, description = "Opportunity published")),
    tag = "Opportunities"
)]
pub async fn publish_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.opp_use_cases.publish(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/pause",
    responses((status = 200, description = "Opportunity paused")),
    tag = "Opportunities"
)]
pub async fn pause_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.opp_use_cases.pause(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/close",
    responses((status = 200, description = "Opportunity closed")),
    tag = "Opportunities"
)]
pub async fn close_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.opp_use_cases.close(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}
