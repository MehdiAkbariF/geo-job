use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::company::{
    AddCompanyLocationCommand, AddMemberCommand, CompanyDto, CompanyMemberDto,
    CreateCompanyCommand, UpdateCompanyCommand,
};
use biz_domain::application::Application;
use biz_domain::opportunity::Opportunity;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/companies",
    request_body = CreateCompanyCommand,
    responses((status = 201, description = "Company registered successfully", body = CompanyDto)),
    tag = "Companies"
)]
pub async fn create_company_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<CreateCompanyCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let company = state.company_use_cases.create_company(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(company)))
}

#[utoipa::path(
    put,
    path = "/api/v1/companies/{id}",
    request_body = UpdateCompanyCommand,
    responses((status = 200, description = "Company updated successfully", body = CompanyDto)),
    tag = "Companies"
)]
pub async fn update_company_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<UpdateCompanyCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let updated = state.company_use_cases.update_company(auth.user_id, id, cmd).await?;
    Ok(Json(updated))
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}",
    responses((status = 200, description = "Company public info", body = CompanyDto)),
    tag = "Companies"
)]
pub async fn get_company_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let company = state.company_use_cases.get_company(id).await?;
    Ok(Json(company))
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/locations",
    request_body = AddCompanyLocationCommand,
    responses((status = 200, description = "Company location linked")),
    tag = "Companies"
)]
pub async fn add_company_location_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<AddCompanyLocationCommand>,
) -> Result<impl IntoResponse, ApiError> {
    state.company_use_cases.add_location(auth.user_id, id, cmd).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/members",
    responses((status = 200, description = "List company team members", body = Vec<CompanyMemberDto>)),
    tag = "Companies"
)]
pub async fn list_members_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let members = state.company_use_cases.list_members(auth.user_id, id).await?;
    Ok(Json(members))
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/members",
    request_body = AddMemberCommand,
    responses((status = 200, description = "Member added to company")),
    tag = "Companies"
)]
pub async fn add_member_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<AddMemberCommand>,
) -> Result<impl IntoResponse, ApiError> {
    state.company_use_cases.add_member(auth.user_id, id, cmd).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/opportunities",
    responses((status = 200, description = "Employer workspace opportunities list", body = Vec<Opportunity>)),
    tag = "Employer ATS"
)]
pub async fn list_company_opportunities_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let opps = state.company_use_cases.list_company_opportunities(auth.user_id, id).await?;
    Ok(Json(opps))
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/public-opportunities",
    responses((status = 200, description = "Public published opportunities of company", body = Vec<Opportunity>)),
    tag = "Companies"
)]
pub async fn list_public_company_opportunities_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let opps = state.company_use_cases.list_public_opportunities(id).await?;
    Ok(Json(opps))
}

#[utoipa::path(
    get,
    path = "/api/v1/opportunities/{id}/applications",
    responses((status = 200, description = "List applicants for this opportunity", body = Vec<Application>)),
    tag = "Employer ATS"
)]
pub async fn list_opportunity_applicants_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let apps = state.app_use_cases.list_by_opportunity(auth.user_id, id).await?;
    Ok(Json(apps))
}