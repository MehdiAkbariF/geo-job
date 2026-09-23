use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Invoice {
    pub id: Uuid,
    pub invoice_number: String,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub tariff_id: Uuid,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub status: String,
    pub payment_method: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}