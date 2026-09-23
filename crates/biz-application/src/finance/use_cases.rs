use super::dto::{CreateInvoiceCommand, InvoiceDto, TariffDto, TransactionDto, WalletDto};
use crate::error::ApplicationError;
use biz_storage::{CompanyRepository, FinanceRepository, StorageError};
use uuid::Uuid;

#[derive(Clone)]
pub struct FinanceUseCases {
    finance_repo: FinanceRepository,
    company_repo: CompanyRepository,
}

impl FinanceUseCases {
    pub fn new(finance_repo: FinanceRepository, company_repo: CompanyRepository) -> Self {
        Self {
            finance_repo,
            company_repo,
        }
    }

    pub async fn get_company_wallet(&self, actor_user_id: Uuid, company_id: Uuid) -> Result<WalletDto, ApplicationError> {
        let _ = self.company_repo.get_user_role(company_id, actor_user_id).await?
            .ok_or_else(|| ApplicationError::Unauthorized("شما عضو این شرکت نیستید".into()))?;

        let w = self.finance_repo.get_or_create_wallet(company_id).await?;
        Ok(WalletDto {
            id: w.id,
            company_id: w.company_id,
            cash_balance: w.cash_balance,
            gift_balance: w.gift_balance,
            total_balance: w.total_balance(),
            currency: w.currency,
            is_frozen: w.is_frozen,
        })
    }

    pub async fn list_tariffs(&self) -> Result<Vec<TariffDto>, ApplicationError> {
        let tariffs = self.finance_repo.list_tariffs().await?;
        Ok(tariffs.into_iter().map(|t| TariffDto {
            id: t.id,
            code: t.code,
            title: t.title,
            description: t.description,
            category: t.category,
            price: t.price,
            gift_credit: t.gift_credit,
            validity_days: t.validity_days,
        }).collect())
    }

    pub async fn create_package_invoice(
        &self,
        actor_user_id: Uuid,
        company_id: Uuid,
        cmd: CreateInvoiceCommand,
    ) -> Result<InvoiceDto, ApplicationError> {
        let role = self.company_repo.get_user_role(company_id, actor_user_id).await?
            .ok_or_else(|| ApplicationError::Unauthorized("شما عضو این سازمان نیستید".into()))?;

        if !role.can_manage_members() {
            return Err(ApplicationError::Unauthorized("فقط مدیران سازمان اجازه خرید بسته اعتباری دارند".into()));
        }

        let tariff = self.finance_repo.find_tariff_by_id(cmd.tariff_id).await?
            .ok_or_else(|| ApplicationError::Validation("تعرفه یا بسته اعتباری یافت نشد".into()))?;

        let inv = self.finance_repo.create_invoice(company_id, actor_user_id, &tariff).await?;
        Ok(InvoiceDto {
            id: inv.id,
            invoice_number: inv.invoice_number,
            subtotal: inv.subtotal,
            tax_amount: inv.tax_amount,
            total_amount: inv.total_amount,
            status: inv.status,
            payment_method: inv.payment_method,
            paid_at: inv.paid_at,
            created_at: inv.created_at,
        })
    }

    /// پرداخت تستی و موفقیت‌آمیز فاکتور شبیه‌ساز (Mock IPG) و شارژ فوری کیف‌پول
    pub async fn mock_pay_invoice(
        &self,
        actor_user_id: Uuid,
        company_id: Uuid,
        invoice_id: Uuid,
    ) -> Result<WalletDto, ApplicationError> {
        let _ = self.company_repo.get_user_role(company_id, actor_user_id).await?
            .ok_or_else(|| ApplicationError::Unauthorized("دسترسی غیرمجاز".into()))?;

        let invoice = self.finance_repo.find_invoice_by_id(invoice_id).await?
            .ok_or_else(|| ApplicationError::Validation("فاکتور یافت نشد".into()))?;

        if invoice.company_id != company_id {
            return Err(ApplicationError::Unauthorized("این فاکتور متعلق به شرکت شما نیست".into()));
        }

        if invoice.status == "paid" {
            return Err(ApplicationError::Validation("این فاکتور قبلاً پرداخت شده است".into()));
        }

        let tariff = self.finance_repo.find_tariff_by_id(invoice.tariff_id).await?
            .ok_or_else(|| ApplicationError::Validation("تعرفه یافت نشد".into()))?;

        // ۱. پرداخت فاکتور
        self.finance_repo.mark_invoice_paid(invoice_id).await?;

        // ۲. شارژ موجودی نقدی و شارژ هدیه در کیف پول
        let _ = self.finance_repo.deposit_package(
            company_id,
            tariff.price,
            tariff.gift_credit,
            invoice.id,
            &tariff.title,
        ).await?;

        let w = self.finance_repo.get_or_create_wallet(company_id).await?;
        Ok(WalletDto {
            id: w.id,
            company_id: w.company_id,
            cash_balance: w.cash_balance,
            gift_balance: w.gift_balance,
            total_balance: w.total_balance(),
            currency: w.currency,
            is_frozen: w.is_frozen,
        })
    }

    pub async fn list_transactions(
        &self,
        actor_user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Vec<TransactionDto>, ApplicationError> {
        let _ = self.company_repo.get_user_role(company_id, actor_user_id).await?
            .ok_or_else(|| ApplicationError::Unauthorized("شما عضو این شرکت نیستید".into()))?;

        let wallet = self.finance_repo.get_or_create_wallet(company_id).await?;
        let list = self.finance_repo.list_wallet_transactions(wallet.id, 50).await?;

        Ok(list.into_iter().map(|t| TransactionDto {
            id: t.id,
            amount: t.amount,
            balance_after: t.balance_after,
            tx_type: t.tx_type,
            reference_type: t.reference_type,
            description: t.description,
            created_at: t.created_at,
        }).collect())
    }

    /// باز کردن اطلاعات تماس کارجو از روی نقشه استعدادها با کسر تعرفه از کیف پول
    pub async fn unlock_candidate_contact(
        &self,
        actor_user_id: Uuid,
        company_id: Uuid,
        candidate_id: Uuid,
    ) -> Result<(), ApplicationError> {
        let role = self.company_repo.get_user_role(company_id, actor_user_id).await?
            .ok_or_else(|| ApplicationError::Unauthorized("عضویت غیرمجاز".into()))?;

        if !role.can_manage_opportunities() {
            return Err(ApplicationError::Unauthorized("دسترسی کافی برای خرید اطلاعات تماس ندارید".into()));
        }

        // اگر قبلاً باز شده باشد، بدون هزینه دوباره تایید می‌شود
        if self.finance_repo.is_candidate_contact_unlocked(company_id, candidate_id).await? {
            return Ok(());
        }

        let tariff = self.finance_repo.find_tariff_by_code("srv_talent_unlock").await?
            .ok_or_else(|| ApplicationError::Validation("تعرفه خرید رزومه تعریف نشده است".into()))?;

        // کسر اتمیک و امن از موجودی
        self.finance_repo.deduct_balance(
            company_id,
            tariff.price,
            "talent_unlock",
            Some(candidate_id),
            &format!("خرید دسترسی به شماره تماس کارجو (کد {})", &candidate_id.to_string()[..8]),
        ).await?;

        self.finance_repo.unlock_candidate_contact(company_id, candidate_id, actor_user_id).await?;
        Ok(())
    }
}