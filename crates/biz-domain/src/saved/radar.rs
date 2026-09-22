use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct JobRadar {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    #[schema(value_type = Option<[f64; 2]>)]
    pub center_coordinates: Option<[f64; 2]>,
    pub radius_meters: i32,
    pub keywords: Option<String>,
    pub min_salary: Option<Decimal>,
    pub workplace_type: Option<String>,
    pub category_id: Option<Uuid>,
    pub is_active: bool,
    pub notify_in_app: bool,
    pub notify_sms: bool,
    pub last_triggered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}