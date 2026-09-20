use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::identity::{LoginCommand, RegisterCommand};
use utoipa::OpenApi;

pub async fn register_handler(
    State(state): State<AppState>,
    Json(cmd): Json<RegisterCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.auth_use_cases.register(cmd).await?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn login_handler(
    State(state): State<AppState>,
    Json(cmd): Json<LoginCommand>,
) -> Result<impl IntoResponse, ApiError> {
    let res = state.auth_use_cases.login(cmd).await?;
    Ok((StatusCode::OK, Json(res)))
}

pub async fn me_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    Ok((StatusCode::OK, Json(serde_json::json!({ "user_id": auth.user_id }))))
}