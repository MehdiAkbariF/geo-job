use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use biz_domain::taxonomy::{Category, Industry, Occupation, Skill};
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SkillQuery {
    pub q: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/taxonomies/categories",
    responses((status = 200, description = "Categories list", body = Vec<Category>)),
    tag = "Taxonomies"
)]
pub async fn list_categories_handler(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let list = state.taxonomy_use_cases.list_categories().await?;
    Ok(Json(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/taxonomies/industries",
    responses((status = 200, description = "Industries list", body = Vec<Industry>)),
    tag = "Taxonomies"
)]
pub async fn list_industries_handler(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let list = state.taxonomy_use_cases.list_industries().await?;
    Ok(Json(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/taxonomies/occupations",
    responses((status = 200, description = "Occupations list", body = Vec<Occupation>)),
    tag = "Taxonomies"
)]
pub async fn list_occupations_handler(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let list = state.taxonomy_use_cases.list_occupations().await?;
    Ok(Json(list))
}

#[utoipa::path(
    get,
    path = "/api/v1/taxonomies/skills/resolve",
    params(SkillQuery),
    responses((status = 200, description = "Resolved skill", body = Option<Skill>)),
    tag = "Taxonomies"
)]
pub async fn resolve_skill_handler(
    State(state): State<AppState>,
    Query(query): Query<SkillQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let skill = state.taxonomy_use_cases.resolve_skill(&query.q).await?;
    Ok(Json(skill))
}