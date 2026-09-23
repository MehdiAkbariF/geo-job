use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Wallet {
    pub id: Uuid,
    pub company_id: Uuid,
    pub cash_balance: Decimal,
    pub gift_balance: Decimal,
    pub currency: String,
    pub is_frozen: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Wallet {
    pub fn total_balance(&self) -> Decimal {
        self.cash_balance + self.gift_balance
    }

    pub fn can_afford(&self, amount: Decimal) -> bool {
        !self.is_frozen && self.total_balance() >= amount
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub amount: Decimal,
    pub cash_amount: Decimal,
    pub gift_amount: Decimal,
    pub balance_after: Decimal,
    pub tx_type: String,
    pub reference_type: String,
    pub reference_id: Option<Uuid>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}