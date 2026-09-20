use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

pub async fn save_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_use_cases.save_opportunity(auth.user_id, id).await?;
    Ok(StatusCode::CREATED)
}

pub async fn remove_saved_opportunity_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.saved_use_cases.remove_saved_opportunity(auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

pub async fn list_saved_opportunities_handler(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let items = state.saved_use_cases.list_saved_opportunities(auth.user_id).await?;
    Ok(Json(items))
}