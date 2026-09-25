use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CompanySummary {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo_storage_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct OpportunitySearchResult {
    pub id: Uuid,
    pub title: String,
    pub description_summary: String,
    pub opportunity_type: String,
    pub workplace_type: String,
    pub remote_scope: Option<String>,
    pub experience_level: String,
    pub salary_min: Option<Decimal>,
    pub salary_max: Option<Decimal>,
    pub salary_currency: String,
    pub salary_period: String,
    pub published_at: Option<DateTime<Utc>>,
    pub company: CompanySummary,
    pub location_summary: Option<String>,
    #[schema(value_type = Option<String>, format = Uuid)]
    pub location_id: Option<Uuid>,
    #[schema(value_type = Option<[f64; 2]>, example = json!([51.3890, 35.6892]))]
    pub coordinates: Option<[f64; 2]>,
    pub distance_meters: Option<f64>,
    pub match_score: Option<u8>,
    pub match_reasons: Vec<String>,
    pub is_urgent: bool,       // استخدام فوری
    pub is_featured: bool,     // سنجاق طلایی نقشه
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SpatialContext {
    pub scope: String,
    pub detected_city: Option<String>,
    pub city_total_jobs: Option<i64>,
    pub national_total_jobs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SearchPageResult {
    pub items: Vec<OpportunitySearchResult>,
    pub total_count: i64,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub spatial_context: SpatialContext,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MapPinSummary {
    pub location_id: Uuid,
    #[schema(value_type = [f64; 2], example = json!([51.3890, 35.7200]))]
    pub coordinates: [f64; 2],
    pub address_summary: Option<String>,
    pub opportunity_count: i64,
    pub top_categories: Vec<String>,
    pub sample_companies: Vec<String>,
    pub min_salary: Option<Decimal>,
    pub max_salary: Option<Decimal>,
    pub salary_currency: String,
}