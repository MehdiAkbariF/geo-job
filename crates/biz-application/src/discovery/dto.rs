use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Default, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchOpportunitiesRequest {
    pub q: Option<String>,
    pub category_id: Option<Uuid>,
    pub occupation_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub skill_ids: Option<String>,
    pub opportunity_type: Option<String>,
    pub workplace_type: Option<String>,
    pub experience_level: Option<String>,
    pub salary_min: Option<Decimal>,
    pub include_remote: Option<bool>,
    pub near_me: Option<bool>,
    /// Specific city filter (e.g. "tehran", "isfahan", "تهران")
    pub city: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_meters: Option<f64>,
    pub bbox: Option<String>,
    pub sort: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetMapPinsRequest {
    pub bbox: Option<String>,
    pub city: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_meters: Option<f64>,
    pub category_id: Option<Uuid>,
    pub workplace_type: Option<String>,
    pub salary_min: Option<Decimal>,
    pub limit: Option<usize>,
}