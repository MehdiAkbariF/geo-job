use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::governance::{
    CompanyVerificationDto, CreateReportCommand, ReviewVerificationCommand,
    SubmitVerificationCommand,
};
use biz_domain::governance::Report;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/reports",
    request_body = CreateReportCommand,
    responses((status = 201, description = "Opportunity reported")),
    tag = "Governance & Trust"
)]
pub async fn report_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(opportunity_id): Path<Uuid>,
    Json(mut cmd): Json<CreateReportCommand>,
) -> Result<impl IntoResponse, ApiError> {
    cmd.opportunity_id = opportunity_id;
    let report_id = state.gov_use_cases.report_opportunity(Some(auth.user_id), cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "report_id": report_id }))))
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/verifications",
    request_body = SubmitVerificationCommand,
    responses((status = 201, description = "Verification submitted")),
    tag = "Governance & Trust"
)]
pub async fn submit_verification_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(company_id): Path<Uuid>,
    Json(mut cmd): Json<SubmitVerificationCommand>,
) -> Result<impl IntoResponse, ApiError> {
    cmd.company_id = company_id;
    let ver_id = state.gov_use_cases.submit_company_verification(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "verification_id": ver_id }))))
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/verifications",
    responses((status = 200, description = "List pending verifications for admin", body = Vec<CompanyVerificationDto>)),
    tag = "Governance & Trust"
)]
pub async fn list_pending_verifications_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.gov_use_cases.list_pending_verifications(auth.user_id).await?;
    Ok(Json(list))
}

#[utoipa::path(
    post,
    path = "/api/v1/admin/verifications/{id}/review",
    request_body = ReviewVerificationCommand,
    responses((status = 200, description = "Verification reviewed")),
    tag = "Governance & Trust"
)]
pub async fn review_verification_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(cmd): Json<ReviewVerificationCommand>,
) -> Result<impl IntoResponse, ApiError> {
    state.gov_use_cases.review_verification(auth.user_id, id, cmd).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/reports",
    responses((status = 200, description = "List all reports for admin", body = Vec<Report>)),
    tag = "Governance & Trust"
)]
pub async fn list_reports_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let reports = state.gov_use_cases.list_reports(auth.user_id).await?;
    Ok(Json(reports))
}