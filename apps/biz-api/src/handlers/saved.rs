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
use biz_domain::saved::SavedSearch;
use uuid::Uuid;

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