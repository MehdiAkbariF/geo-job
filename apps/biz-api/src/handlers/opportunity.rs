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
use biz_domain::opportunity::Opportunity;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/opportunities",
    request_body = CreateOpportunityCommand,
    responses((status = 201, description = "Opportunity created", body = Opportunity)),
    tag = "Opportunities"
)]
pub async fn create_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(mut cmd): Json<CreateOpportunityCommand>,
) -> Result<impl IntoResponse, ApiError> {
    cmd.company_id = id;
    let opp = state.opp_use_cases.create_opportunity(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(opp)))
}

#[utoipa::path(
    get,
    path = "/api/v1/opportunities/{id}",
    responses((status = 200, description = "Opportunity details", body = Opportunity)),
    tag = "Opportunities"
)]
pub async fn get_opportunity_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let opp = state.discovery_repo_pool().find_by_id(id).await?
        .ok_or(biz_storage::StorageError::UserNotFound)?;
    Ok(Json(opp))
}

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/track-click",
    responses((status = 200, description = "External click tracked")),
    tag = "Opportunities"
)]
pub async fn track_opportunity_click_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let _ = state.discovery_repo_pool().find_by_id(id).await?
        .ok_or(biz_storage::StorageError::UserNotFound)?;
    tracing::info!("Tracked external apply click for opportunity: {}", id);
    Ok(StatusCode::OK)
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
    path = "/api/v1/opportunities/{id}/resume",
    responses((status = 200, description = "Opportunity resumed")),
    tag = "Opportunities"
)]
pub async fn resume_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.opp_use_cases.resume(auth.user_id, id).await?;
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

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/ladder",
    responses((status = 200, description = "Opportunity laddered on map via wallet deduction")),
    tag = "Opportunities"
)]
pub async fn ladder_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.opp_use_cases.ladder(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/feature-pin",
    responses((status = 200, description = "Featured golden pin enabled on map via wallet deduction")),
    tag = "Opportunities"
)]
pub async fn feature_opportunity_pin_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.opp_use_cases.feature_pin(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}