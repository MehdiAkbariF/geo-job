use crate::error::ApiError;
use crate::extractors::auth::AuthenticatedUser;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use biz_application::governance::CreateReportCommand;
use uuid::Uuid;

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