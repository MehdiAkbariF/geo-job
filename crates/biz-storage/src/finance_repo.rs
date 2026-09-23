use crate::error::StorageError;
use biz_domain::finance::{Invoice, Tariff, Wallet, WalletTransaction};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct WalletDbRow {
    id: Uuid,
    company_id: Uuid,
    cash_balance: Decimal,
    gift_balance: Decimal,
    currency: String,
    is_frozen: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WalletDbRow {
    fn to_domain(self) -> Wallet {
        Wallet {
            id: self.id,
            company_id: self.company_id,
            cash_balance: self.cash_balance,
            gift_balance: self.gift_balance,
            currency: self.currency,
            is_frozen: self.is_frozen,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct TxDbRow {
    id: Uuid,
    wallet_id: Uuid,
    amount: Decimal,
    cash_amount: Decimal,
    gift_amount: Decimal,
    balance_after: Decimal,
    tx_type: String,
    reference_type: String,
    reference_id: Option<Uuid>,
    description: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct TariffDbRow {
    id: Uuid,
    code: String,
    title: String,
    description: Option<String>,
    category: String,
    price: Decimal,
    gift_credit: Decimal,
    validity_days: Option<i32>,
    is_active: bool,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct InvoiceDbRow {
    id: Uuid,
    invoice_number: String,
    company_id: Uuid,
    user_id: Uuid,
    tariff_id: Uuid,
    subtotal: Decimal,
    tax_amount: Decimal,
    total_amount: Decimal,
    status: String,
    payment_method: String,
    paid_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FinanceRepository {
    pool: PgPool,
}

impl FinanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// دریافت یا ایجاد خودکار کیف پول برای شرکت
    pub async fn get_or_create_wallet(&self, company_id: Uuid) -> Result<Wallet, StorageError> {
        let sql = r#"
            INSERT INTO wallets (company_id, cash_balance, gift_balance, currency)
            VALUES ($1, 0.00, 0.00, 'IRR')
            ON CONFLICT (company_id) DO UPDATE SET updated_at = NOW()
            RETURNING *
        "#;

        let row = sqlx::query_as::<_, WalletDbRow>(sql)
            .bind(company_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.to_domain())
    }

    /// کسر اتمیک و ضدگلوله از کیف پول با قفل سطری FOR UPDATE
    pub async fn deduct_balance(
        &self,
        company_id: Uuid,
        required_amount: Decimal,
        ref_type: &str,
        ref_id: Option<Uuid>,
        description: &str,
    ) -> Result<WalletTransaction, StorageError> {
        let mut tx = self.pool.begin().await?;

        // ۱. قفل انحصاری سطر کیف پول جهت جلوگیری از Race Condition
        let select_sql = "SELECT * FROM wallets WHERE company_id = $1 FOR UPDATE";
        let wallet = sqlx::query_as::<_, WalletDbRow>(select_sql)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await?;

        if wallet.is_frozen {
            return Err(StorageError::Security("کیف پول این سازمان مسدود است".into()));
        }

        let total = wallet.cash_balance + wallet.gift_balance;
        if total < required_amount {
            return Err(StorageError::Validation(format!(
                "موجودی ناکافی است. موجودی کل: {} ریال، مبلغ مورد نیاز: {} ریال",
                total, required_amount
            )));
        }

        // اولویت کسر با شارژ هدیه (غیرقابل استرداد) و سپس موجودی نقدی
        let gift_deduct = wallet.gift_balance.min(required_amount);
        let cash_deduct = required_amount - gift_deduct;

        let new_gift = wallet.gift_balance - gift_deduct;
        let new_cash = wallet.cash_balance - cash_deduct;
        let balance_after = new_cash + new_gift;

        // ۲. به‌روزرسانی کیف پول
        let update_sql = "UPDATE wallets SET cash_balance = $1, gift_balance = $2, updated_at = NOW() WHERE id = $3";
        sqlx::query(update_sql)
            .bind(new_cash)
            .bind(new_gift)
            .bind(wallet.id)
            .execute(&mut *tx)
            .await?;

        // ۳. ثبت رکورد تغییر ناپذیر در دفتر کل تراکنش‌ها
        let tx_sql = r#"
            INSERT INTO wallet_transactions (
                wallet_id, amount, cash_amount, gift_amount, balance_after,
                tx_type, reference_type, reference_id, description
            )
            VALUES ($1, $2, $3, $4, $5, 'withdraw', $6, $7, $8)
            RETURNING *
        "#;

        let tx_row = sqlx::query_as::<_, TxDbRow>(tx_sql)
            .bind(wallet.id)
            .bind(-required_amount)
            .bind(-cash_deduct)
            .bind(-gift_deduct)
            .bind(balance_after)
            .bind(ref_type)
            .bind(ref_id)
            .bind(description)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(WalletTransaction {
            id: tx_row.id,
            wallet_id: tx_row.wallet_id,
            amount: tx_row.amount,
            cash_amount: tx_row.cash_amount,
            gift_amount: tx_row.gift_amount,
            balance_after: tx_row.balance_after,
            tx_type: tx_row.tx_type,
            reference_type: tx_row.reference_type,
            reference_id: tx_row.reference_id,
            description: tx_row.description,
            created_at: tx_row.created_at,
        })
    }

    /// شارژ مستقیم بسته اعتباری به همراه اعتبار هدیه در کیف پول
    pub async fn deposit_package(
        &self,
        company_id: Uuid,
        cash_credit: Decimal,
        gift_credit: Decimal,
        invoice_id: Uuid,
        package_name: &str,
    ) -> Result<WalletTransaction, StorageError> {
        let mut tx = self.pool.begin().await?;

        let select_sql = "SELECT * FROM wallets WHERE company_id = $1 FOR UPDATE";
        let wallet = sqlx::query_as::<_, WalletDbRow>(select_sql)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await?;

        let new_cash = wallet.cash_balance + cash_credit;
        let new_gift = wallet.gift_balance + gift_credit;
        let balance_after = new_cash + new_gift;
        let total_deposit = cash_credit + gift_credit;

        let update_sql = "UPDATE wallets SET cash_balance = $1, gift_balance = $2, updated_at = NOW() WHERE id = $3";
        sqlx::query(update_sql)
            .bind(new_cash)
            .bind(new_gift)
            .bind(wallet.id)
            .execute(&mut *tx)
            .await?;

        let desc = format!("خرید و شارژ {}: موجودی نقدی +{} و هدیه +{}", package_name, cash_credit, gift_credit);
        let tx_sql = r#"
            INSERT INTO wallet_transactions (
                wallet_id, amount, cash_amount, gift_amount, balance_after,
                tx_type, reference_type, reference_id, description
            )
            VALUES ($1, $2, $3, $4, $5, 'deposit', 'package_charge', $6, $7)
            RETURNING *
        "#;

        let tx_row = sqlx::query_as::<_, TxDbRow>(tx_sql)
            .bind(wallet.id)
            .bind(total_deposit)
            .bind(cash_credit)
            .bind(gift_credit)
            .bind(balance_after)
            .bind(invoice_id)
            .bind(&desc)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(WalletTransaction {
            id: tx_row.id,
            wallet_id: tx_row.wallet_id,
            amount: tx_row.amount,
            cash_amount: tx_row.cash_amount,
            gift_amount: tx_row.gift_amount,
            balance_after: tx_row.balance_after,
            tx_type: tx_row.tx_type,
            reference_type: tx_row.reference_type,
            reference_id: tx_row.reference_id,
            description: tx_row.description,
            created_at: tx_row.created_at,
        })
    }

    pub async fn list_tariffs(&self) -> Result<Vec<Tariff>, StorageError> {
        let sql = "SELECT * FROM tariffs WHERE is_active = true ORDER BY category, price ASC";
        let rows = sqlx::query_as::<_, TariffDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| Tariff {
            id: r.id,
            code: r.code,
            title: r.title,
            description: r.description,
            category: r.category,
            price: r.price,
            gift_credit: r.gift_credit,
            validity_days: r.validity_days,
            is_active: r.is_active,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn find_tariff_by_id(&self, id: Uuid) -> Result<Option<Tariff>, StorageError> {
        let sql = "SELECT * FROM tariffs WHERE id = $1";
        let row = sqlx::query_as::<_, TariffDbRow>(sql).bind(id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Tariff {
            id: r.id,
            code: r.code,
            title: r.title,
            description: r.description,
            category: r.category,
            price: r.price,
            gift_credit: r.gift_credit,
            validity_days: r.validity_days,
            is_active: r.is_active,
            created_at: r.created_at,
        }))
    }

    pub async fn find_tariff_by_code(&self, code: &str) -> Result<Option<Tariff>, StorageError> {
        let sql = "SELECT * FROM tariffs WHERE code = $1";
        let row = sqlx::query_as::<_, TariffDbRow>(sql).bind(code).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Tariff {
            id: r.id,
            code: r.code,
            title: r.title,
            description: r.description,
            category: r.category,
            price: r.price,
            gift_credit: r.gift_credit,
            validity_days: r.validity_days,
            is_active: r.is_active,
            created_at: r.created_at,
        }))
    }

    pub async fn create_invoice(
        &self,
        company_id: Uuid,
        user_id: Uuid,
        tariff: &Tariff,
    ) -> Result<Invoice, StorageError> {
        let inv_num = format!("INV-{}-{}", Utc::now().format("%Y%m%d"), &Uuid::new_v4().to_string()[..6]);
        let tax = tariff.price * Decimal::new(10, 2); // ۱۰٪ مالیات بر ارزش افزوده
        let total = tariff.price + tax;

        let sql = r#"
            INSERT INTO invoices (
                invoice_number, company_id, user_id, tariff_id,
                subtotal, tax_amount, total_amount, status, payment_method
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', 'mock_gateway')
            RETURNING *
        "#;

        let row = sqlx::query_as::<_, InvoiceDbRow>(sql)
            .bind(inv_num)
            .bind(company_id)
            .bind(user_id)
            .bind(tariff.id)
            .bind(tariff.price)
            .bind(tax)
            .bind(total)
            .fetch_one(&self.pool)
            .await?;

        Ok(Invoice {
            id: row.id,
            invoice_number: row.invoice_number,
            company_id: row.company_id,
            user_id: row.user_id,
            tariff_id: row.tariff_id,
            subtotal: row.subtotal,
            tax_amount: row.tax_amount,
            total_amount: row.total_amount,
            status: row.status,
            payment_method: row.payment_method,
            paid_at: row.paid_at,
            created_at: row.created_at,
        })
    }

    pub async fn find_invoice_by_id(&self, invoice_id: Uuid) -> Result<Option<Invoice>, StorageError> {
        let sql = "SELECT * FROM invoices WHERE id = $1";
        let row = sqlx::query_as::<_, InvoiceDbRow>(sql).bind(invoice_id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| Invoice {
            id: r.id,
            invoice_number: r.invoice_number,
            company_id: r.company_id,
            user_id: r.user_id,
            tariff_id: r.tariff_id,
            subtotal: r.subtotal,
            tax_amount: r.tax_amount,
            total_amount: r.total_amount,
            status: r.status,
            payment_method: r.payment_method,
            paid_at: r.paid_at,
            created_at: r.created_at,
        }))
    }

    pub async fn mark_invoice_paid(&self, invoice_id: Uuid) -> Result<(), StorageError> {
        let sql = "UPDATE invoices SET status = 'paid', paid_at = NOW() WHERE id = $1";
        sqlx::query(sql).bind(invoice_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_wallet_transactions(&self, wallet_id: Uuid, limit: i64) -> Result<Vec<WalletTransaction>, StorageError> {
        let sql = "SELECT * FROM wallet_transactions WHERE wallet_id = $1 ORDER BY created_at DESC LIMIT $2";
        let rows = sqlx::query_as::<_, TxDbRow>(sql).bind(wallet_id).bind(limit).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| WalletTransaction {
            id: r.id,
            wallet_id: r.wallet_id,
            amount: r.amount,
            cash_amount: r.cash_amount,
            gift_amount: r.gift_amount,
            balance_after: r.balance_after,
            tx_type: r.tx_type,
            reference_type: r.reference_type,
            reference_id: r.reference_id,
            description: r.description,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn unlock_candidate_contact(&self, company_id: Uuid, candidate_id: Uuid, user_id: Uuid) -> Result<(), StorageError> {
        let sql = "INSERT INTO candidate_contact_unlocks (company_id, candidate_id, unlocked_by_user_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING";
        sqlx::query(sql).bind(company_id).bind(candidate_id).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn is_candidate_contact_unlocked(&self, company_id: Uuid, candidate_id: Uuid) -> Result<bool, StorageError> {
        let sql = "SELECT EXISTS(SELECT 1 FROM candidate_contact_unlocks WHERE company_id = $1 AND candidate_id = $2)";
        let exists: bool = sqlx::query_scalar(sql).bind(company_id).bind(candidate_id).fetch_one(&self.pool).await?;
        Ok(exists)
    }
}