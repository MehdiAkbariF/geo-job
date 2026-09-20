use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::identity::{LoginCommand, RegisterCommand};

#[utoipa::path(
    post,
    path = "/api/v1/auth/register",
    responses((status = 201, description = "User registered successfully")),
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
    responses((status = 200, description = "Login successful")),
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
