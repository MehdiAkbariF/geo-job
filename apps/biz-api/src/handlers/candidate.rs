use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::candidate::{
    AddExperienceCommand, CandidatePreferencesDto, CandidateProfileDto, SetSkillsCommand,
    TrackedApplicationDto, UpdateProfileCommand,
};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/v1/candidates/me",
    responses((status = 200, description = "Current candidate full profile", body = CandidateProfileDto)),
    tag = "Candidate Profile"
)]
pub async fn get_my_profile_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let profile = state.candidate_use_cases.get_full_profile(auth.user_id).await?;
    Ok(Json(profile))
}

#[utoipa::path(
    put,
    path = "/api/v1/candidates/me",
    request_body = UpdateProfileCommand,
    responses((status = 200, description = "Profile updated", body = CandidateProfileDto)),
    tag = "Candidate Profile"
)]
pub async fn update_my_profile_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<UpdateProfileCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let updated = state.candidate_use_cases.update_profile(auth.user_id, cmd).await?;
    Ok(Json(updated))
}

#[utoipa::path(
    post,
    path = "/api/v1/candidates/me/experiences",
    request_body = AddExperienceCommand,
    responses((status = 201, description = "Experience added")),
    tag = "Candidate Profile"
)]
pub async fn add_experience_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<AddExperienceCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let id = state.candidate_use_cases.add_experience(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "experience_id": id }))))
}

#[utoipa::path(
    delete,
    path = "/api/v1/candidates/me/experiences/{id}",
    responses((status = 200, description = "Experience deleted")),
    tag = "Candidate Profile"
)]
pub async fn delete_experience_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.candidate_use_cases.delete_experience(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    put,
    path = "/api/v1/candidates/me/skills",
    request_body = SetSkillsCommand,
    responses((status = 200, description = "Skills updated")),
    tag = "Candidate Profile"
)]
pub async fn set_skills_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<SetSkillsCommand>,
) -> Result<impl IntoResponse, ApiError> {
    state.candidate_use_cases.set_skills(auth.user_id, &cmd.skill_ids).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/candidates/me/preferences",
    responses((status = 200, description = "Candidate preferences", body = CandidatePreferencesDto)),
    tag = "Candidate Profile"
)]
pub async fn get_preferences_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let prefs = state.candidate_use_cases.get_preferences(auth.user_id).await?;
    Ok(Json(prefs))
}

#[utoipa::path(
    put,
    path = "/api/v1/candidates/me/preferences",
    request_body = CandidatePreferencesDto,
    responses((status = 200, description = "Candidate preferences saved")),
    tag = "Candidate Profile"
)]
pub async fn set_preferences_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(dto): Json<CandidatePreferencesDto>,
) -> Result<impl IntoResponse, ApiError> {
    state.candidate_use_cases.set_preferences(auth.user_id, dto).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/candidates/me/applications",
    responses((status = 200, description = "Track my applications", body = Vec<TrackedApplicationDto>)),
    tag = "Candidate Profile"
)]
pub async fn list_my_applications_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.candidate_use_cases.list_my_applications(auth.user_id).await?;
    Ok(Json(list))
}