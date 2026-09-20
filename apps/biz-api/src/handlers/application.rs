use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::application::{
    ApplicationDossierDto, ApplicationDto, ChangeApplicationStatusCommand,
    SubmitApplicationCommand,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/v1/opportunities/{id}/applications",
    request_body = SubmitApplicationCommand,
    responses((status = 201, description = "Application submitted", body = ApplicationDto)),
    tag = "Applications"
)]
pub async fn submit_application_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(opportunity_id): Path<Uuid>,
    Json(mut cmd): Json<SubmitApplicationCommand>,
) -> Result<impl IntoResponse, ApiError> {
    cmd.opportunity_id = opportunity_id;
    let app = state.app_use_cases.submit_application(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(app)))
}

#[utoipa::path(
    post,
    path = "/api/v1/applications/{id}/status",
    request_body = ChangeApplicationStatusCommand,
    responses((status = 200, description = "Status changed")),
    tag = "Applications"
)]
pub async fn change_application_status_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(application_id): Path<Uuid>,
    Json(cmd): Json<ChangeApplicationStatusCommand>,
) -> Result<impl IntoResponse, ApiError> {
    state.app_use_cases.change_status(auth.user_id, application_id, cmd).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}",
    responses((status = 200, description = "Full applicant dossier for employers", body = ApplicationDossierDto)),
    tag = "Employer ATS"
)]
pub async fn get_application_dossier_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let dossier = state.app_use_cases.get_application_dossier(auth.user_id, id).await?;
    Ok(Json(dossier))
}