use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SearchOpportunitiesRequest {
    pub q: Option<String>,
    pub category_id: Option<Uuid>,
    pub occupation_id: Option<Uuid>,
    pub company_id: Option<Uuid>,
    pub opportunity_type: Option<String>,
    pub workplace_type: Option<String>,
    pub experience_level: Option<String>,
    pub salary_min: Option<Decimal>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub radius_meters: Option<f64>,
    pub bbox: Option<String>, // "west,south,east,north"
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}