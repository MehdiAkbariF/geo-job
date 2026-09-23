use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Tariff {
    pub id: Uuid,
    pub code: String,
    pub title: String,
    pub description: Option<String>,
    pub category: String, // credit_package, single_service, add_on
    pub price: Decimal,
    pub gift_credit: Decimal,
    pub validity_days: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}