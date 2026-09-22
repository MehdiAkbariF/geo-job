use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::candidate::{
    AddEducationCommand, AddExperienceCommand, AddLanguageCommand, AddReferenceCommand,
    AddResumeCommand, CandidatePreferencesDto, CandidateProfileDto, SearchTalentsRequest,
    SendInvitationCommand, SetSkillsCommand, TrackedApplicationDto, UpdateProfileCommand,
};
use biz_domain::candidate::TalentSearchResult;
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

// نقشه استعدادها و جستجوی کارجویان برای کارفرما
#[utoipa::path(
    get,
    path = "/api/v1/talents/search",
    params(SearchTalentsRequest),
    responses((status = 200, description = "Search talents on map for employers", body = Vec<TalentSearchResult>)),
    tag = "Employer ATS"
)]
pub async fn search_talents_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Query(req): Query<SearchTalentsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let talents = state.candidate_use_cases.search_talents_for_employer(auth.user_id, req).await?;
    Ok(Json(talents))
}

// کارجویان پیشنهادی متناسب با یک آگهی شغلی خاص
#[utoipa::path(
    get,
    path = "/api/v1/opportunities/{id}/matched-candidates",
    responses((status = 200, description = "Candidates matching this specific opportunity", body = Vec<TalentSearchResult>)),
    tag = "Employer ATS"
)]
pub async fn get_matched_candidates_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let list = state.candidate_use_cases.get_matched_talents_for_opportunity(auth.user_id, id).await?;
    Ok(Json(list))
}

// ارسال دعوت‌نامه رسمی کارفرما به کارجو
#[utoipa::path(
    post,
    path = "/api/v1/candidates/{id}/invite",
    request_body = SendInvitationCommand,
    responses((status = 201, description = "Job invitation sent")),
    tag = "Employer ATS"
)]
pub async fn invite_candidate_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(candidate_id): Path<Uuid>,
    Json(cmd): Json<SendInvitationCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let inv_id = state.candidate_use_cases.send_job_invitation(auth.user_id, candidate_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "invitation_id": inv_id }))))
}

// تجربیات کاری
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

// سوابق تحصیلی
#[utoipa::path(
    post,
    path = "/api/v1/candidates/me/educations",
    request_body = AddEducationCommand,
    responses((status = 201, description = "Education added")),
    tag = "Candidate Profile"
)]
pub async fn add_education_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<AddEducationCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let id = state.candidate_use_cases.add_education(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "education_id": id }))))
}

#[utoipa::path(
    delete,
    path = "/api/v1/candidates/me/educations/{id}",
    responses((status = 200, description = "Education deleted")),
    tag = "Candidate Profile"
)]
pub async fn delete_education_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.candidate_use_cases.delete_education(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

// زبان‌های خارجی
#[utoipa::path(
    post,
    path = "/api/v1/candidates/me/languages",
    request_body = AddLanguageCommand,
    responses((status = 201, description = "Language added")),
    tag = "Candidate Profile"
)]
pub async fn add_language_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<AddLanguageCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let id = state.candidate_use_cases.add_language(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "language_id": id }))))
}

#[utoipa::path(
    delete,
    path = "/api/v1/candidates/me/languages/{id}",
    responses((status = 200, description = "Language deleted")),
    tag = "Candidate Profile"
)]
pub async fn delete_language_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.candidate_use_cases.delete_language(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

// معرف‌ها
#[utoipa::path(
    post,
    path = "/api/v1/candidates/me/references",
    request_body = AddReferenceCommand,
    responses((status = 201, description = "Reference added")),
    tag = "Candidate Profile"
)]
pub async fn add_reference_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<AddReferenceCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let id = state.candidate_use_cases.add_reference(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "reference_id": id }))))
}

#[utoipa::path(
    delete,
    path = "/api/v1/candidates/me/references/{id}",
    responses((status = 200, description = "Reference deleted")),
    tag = "Candidate Profile"
)]
pub async fn delete_reference_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.candidate_use_cases.delete_reference(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

// رزومه‌ها
#[utoipa::path(
    post,
    path = "/api/v1/candidates/me/resumes",
    request_body = AddResumeCommand,
    responses((status = 201, description = "Resume registered")),
    tag = "Candidate Profile"
)]
pub async fn add_resume_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<AddResumeCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let id = state.candidate_use_cases.add_resume(auth.user_id, cmd).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "resume_id": id }))))
}

#[utoipa::path(
    delete,
    path = "/api/v1/candidates/me/resumes/{id}",
    responses((status = 200, description = "Resume deleted")),
    tag = "Candidate Profile"
)]
pub async fn delete_resume_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.candidate_use_cases.delete_resume(auth.user_id, id).await?;
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