use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::identity::{
    AuthResponse, LoginCommand, LogoutCommand, OnboardingCommand, OtpAuthResponse,
    RefreshTokenCommand, RegisterCommand, SendOtpCommand, UserContextDto, VerifyOtpCommand,
};

pub type RefreshTokenRequest = RefreshTokenCommand;
#[utoipa::path(
    post,
    path = "/api/v1/auth/otp/send",
    request_body = SendOtpCommand,
    responses((status = 200, description = "OTP code sent successfully")),
    tag = "Auth"
)]
pub async fn send_otp_handler(
    State(state): State<AppState>,
    Json(cmd): Json<SendOtpCommand>,
) -> Result<impl IntoResponse, ApiError> {
    state.auth_use_cases.send_otp(cmd).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({ "message": "کد تایید ارسال شد" }))))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/otp/verify",
    request_body = VerifyOtpCommand,
    responses((status = 200, description = "Phone verified and session issued", body = OtpAuthResponse)),
    tag = "Auth"
)]
pub async fn verify_otp_handler(
    State(state): State<AppState>,
    Json(cmd): Json<VerifyOtpCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let auth = state.auth_use_cases.verify_otp(cmd).await?;
    Ok(Json(auth))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/onboarding",
    request_body = OnboardingCommand,
    responses((status = 200, description = "User persona onboarding completed", body = UserContextDto)),
    tag = "Auth"
)]
pub async fn onboarding_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Json(cmd): Json<OnboardingCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let context = state.auth_use_cases.complete_onboarding(auth.user_id, cmd).await?;
    Ok(Json(context))
}

#[utoipa::path(
    get,
    path = "/api/v1/me",
    responses((status = 200, description = "Current user full context, role and active profile", body = UserContextDto)),
    tag = "Auth"
)]
pub async fn me_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let ctx = state.auth_use_cases.get_me(auth.user_id).await?;
    Ok(Json(ctx))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    request_body = RegisterCommand,
    responses((status = 201, description = "User registered successfully", body = AuthResponse)),
    tag = "Auth"
)]
pub async fn register_handler(
    State(state): State<AppState>,
    Json(cmd): Json<RegisterCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let auth = state.auth_use_cases.register(cmd).await?;
    Ok((StatusCode::CREATED, Json(auth)))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginCommand,
    responses((status = 200, description = "Login successful", body = AuthResponse)),
    tag = "Auth"
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(cmd): Json<LoginCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let auth = state.auth_use_cases.login(cmd).await?;
    Ok(Json(auth))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshTokenCommand,
    responses((status = 200, description = "Tokens refreshed", body = AuthResponse)),
    tag = "Auth"
)]
pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(cmd): Json<RefreshTokenCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let auth = state.auth_use_cases.refresh_tokens(cmd).await?;
    Ok(Json(auth))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = LogoutCommand,
    responses((status = 200, description = "Logged out successfully")),
    tag = "Auth"
)]
pub async fn logout_handler(
    State(state): State<AppState>,
    Json(cmd): Json<LogoutCommand>,
) -> Result<impl IntoResponse, ApiError> {
    state.auth_use_cases.logout(cmd).await?;
    Ok(StatusCode::OK)
}