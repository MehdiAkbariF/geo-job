use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::identity::{AuthResponseDto, LoginCommand, RegisterCommand};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterCommand,
    responses((status = 201, description = "User registered successfully", body = AuthResponseDto)),
    tag = "Auth"
)]
pub async fn register_handler(
    State(state): State<AppState>,
    Json(cmd): Json<RegisterCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.auth_use_cases.register(cmd).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginCommand,
    responses((status = 200, description = "Login successful", body = AuthResponseDto)),
    tag = "Auth"
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(cmd): Json<LoginCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.auth_use_cases.login(cmd).await?;
    Ok((StatusCode::OK, Json(res)))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenRequest,
    responses((status = 200, description = "Tokens refreshed", body = AuthResponseDto)),
    tag = "Auth"
)]
pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.auth_use_cases.refresh_tokens(&req.refresh_token).await?;
    Ok(Json(res))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = RefreshTokenRequest,
    responses((status = 200, description = "Logged out successfully")),
    tag = "Auth"
)]
pub async fn logout_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    state.auth_use_cases.logout(&req.refresh_token).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/v1/me",
    responses((status = 200, description = "Current user info")),
    tag = "Auth"
)]
pub async fn me_handler(
    State(_state): State<AppState>,
    auth: crate::extractors::auth::AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    Ok((StatusCode::OK, Json(serde_json::json!({ "user_id": auth.user_id }))))
}