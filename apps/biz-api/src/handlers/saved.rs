use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::saved::SaveSearchCommand;
use biz_domain::discovery::{CompanySummary, OpportunitySearchResult};
use biz_domain::saved::{JobRadar, Notification, SavedSearch};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// ==========================================
// ۱. هندلرهای بوک‌مارک آگهی و شرکت و جستجوها
// ==========================================

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/save",
    responses((status = 201, description = "Opportunity saved")),
    tag = "Saved"
)]
pub async fn save_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_use_cases.save_opportunity(auth.user_id, id).await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/api/v1/opportunities/{id}/save",
    responses((status = 200, description = "Opportunity unsaved")),
    tag = "Saved"
)]
pub async fn remove_saved_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_use_cases.remove_saved_opportunity(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/me/saved-opportunities",
    responses((status = 200, description = "List saved opportunities", body = Vec<OpportunitySearchResult>)),
    tag = "Saved"
)]
pub async fn list_saved_opportunities_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.saved_use_cases.list_saved_opportunities(auth.user_id).await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/save",
    responses((status = 201, description = "Company saved")),
    tag = "Saved"
)]
pub async fn save_company_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_use_cases.save_company(auth.user_id, id).await?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    delete,
    path = "/api/v1/companies/{id}/save",
    responses((status = 200, description = "Company unsaved")),
    tag = "Saved"
)]
pub async fn remove_saved_company_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_use_cases.remove_saved_company(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/me/saved-companies",
    responses((status = 200, description = "List saved companies", body = Vec<CompanySummary>)),
    tag = "Saved"
)]
pub async fn list_saved_companies_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.saved_use_cases.list_saved_companies(auth.user_id).await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/v1/me/saved-searches",
    request_body = SaveSearchCommand,
    responses((status = 201, description = "Search saved")),
    tag = "Saved"
)]
pub async fn save_search_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<SaveSearchCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let id = state.saved_use_cases.save_search(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "saved_search_id": id }))))
}

#[utoipa::path(
    get,
    path = "/api/v1/me/saved-searches",
    responses((status = 200, description = "List saved searches", body = Vec<SavedSearch>)),
    tag = "Saved"
)]
pub async fn list_saved_searches_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let searches = state.saved_use_cases.list_saved_searches(auth.user_id).await?;
    Ok(Json(searches))
}

// ==========================================
// ۲. هندلرهای رادارهای هوشمند و اعلان‌ها
// ==========================================

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRadarRequest {
    pub title: String,
    pub center_coordinates: Option<[f64; 2]>,
    pub radius_meters: Option<i32>,
    pub keywords: Option<String>,
    pub min_salary: Option<Decimal>,
    pub workplace_type: Option<String>,
    pub category_id: Option<Uuid>,
}

#[utoipa::path(
    post,
    path = "/api/v1/me/radars",
    request_body = CreateRadarRequest,
    responses((status = 201, description = "Spatial job radar created")),
    tag = "Saved"
)]
pub async fn create_radar_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(req): Json<CreateRadarRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let id = state.saved_repo.create_radar(
        auth.user_id,
        &req.title,
        req.center_coordinates,
        req.radius_meters.unwrap_or(3000),
        req.keywords.as_deref(),
        req.min_salary,
        req.workplace_type.as_deref(),
        req.category_id,
    ).await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "radar_id": id }))))
}

#[utoipa::path(
    get,
    path = "/api/v1/me/radars",
    responses((status = 200, description = "List user spatial radars", body = Vec<JobRadar>)),
    tag = "Saved"
)]
pub async fn list_my_radars_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.saved_repo.list_user_radars(auth.user_id).await?;
    Ok(Json(list))
}

#[utoipa::path(
    delete,
    path = "/api/v1/me/radars/{id}",
    responses((status = 200, description = "Radar deleted")),
    tag = "Saved"
)]
pub async fn delete_radar_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_repo.delete_radar(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/me/notifications",
    responses((status = 200, description = "List in-app notifications", body = Vec<Notification>)),
    tag = "Notifications"
)]
pub async fn list_my_notifications_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.saved_repo.list_user_notifications(auth.user_id, 30).await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/v1/me/notifications/{id}/read",
    responses((status = 200, description = "Notification marked as read")),
    tag = "Notifications"
)]
pub async fn mark_notification_read_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_repo.mark_notification_read(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/me/notifications/unread-count",
    responses((status = 200, description = "Get unread count for badge", body = i64)),
    tag = "Notifications"
)]
pub async fn get_unread_notifications_count_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let count = state.saved_repo.get_unread_notifications_count(auth.user_id).await?;
    Ok(Json(serde_json::json!({ "unread_count": count })))
}