use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WalletDto {
    pub id: Uuid,
    pub company_id: Uuid,
    pub cash_balance: Decimal,
    pub gift_balance: Decimal,
    pub total_balance: Decimal,
    pub currency: String,
    pub is_frozen: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TariffDto {
    pub id: Uuid,
    pub code: String,
    pub title: String,
    pub description: Option<String>,
    pub category: String,
    pub price: Decimal,
    pub gift_credit: Decimal,
    pub validity_days: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateInvoiceCommand {
    pub tariff_id: Uuid,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct InvoiceDto {
    pub id: Uuid,
    pub invoice_number: String,
    pub subtotal: Decimal,
    pub tax_amount: Decimal,
    pub total_amount: Decimal,
    pub status: String,
    pub payment_method: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TransactionDto {
    pub id: Uuid,
    pub amount: Decimal,
    pub balance_after: Decimal,
    pub tx_type: String,
    pub reference_type: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}