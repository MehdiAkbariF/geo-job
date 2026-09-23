use super::dto::CreateOpportunityCommand;
use crate::error::ApplicationError;
use biz_domain::opportunity::{NewOpportunity, Opportunity, OpportunityStatus};
use biz_storage::{CompanyRepository, FinanceRepository, OpportunityRepository, StorageError};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct OpportunityUseCases {
    opp_repo: OpportunityRepository,
    company_repo: CompanyRepository,
    finance_repo: FinanceRepository,
}

impl OpportunityUseCases {
    pub fn new(
        opp_repo: OpportunityRepository, 
        company_repo: CompanyRepository,
        finance_repo: FinanceRepository,
    ) -> Self {
        Self { 
            opp_repo, 
            company_repo,
            finance_repo,
        }
    }

    /// ایجاد پیش‌نویس آگهی با مشخصات محلی و اصناف
    pub async fn create_opportunity(
        &self,
        actor_user_id: Uuid,
        cmd: CreateOpportunityCommand,
    ) -> Result<Opportunity, ApplicationError> {
        let role = self
            .company_repo
            .get_user_role(cmd.company_id, actor_user_id)
            .await?
            .ok_or_else(|| ApplicationError::Unauthorized("شما عضو این سازمان یا کسب‌وکار نیستید".into()))?;

        if !role.can_manage_opportunities() {
            return Err(ApplicationError::Unauthorized(
                "دسترسی کافی برای ثبت فرصت شغلی ندارید".into(),
            ));
        }

        let new_opp = NewOpportunity {
            company_id: cmd.company_id,
            title: cmd.title,
            description: cmd.description,
            category_id: cmd.category_id,
            occupation_id: cmd.occupation_id,
            opportunity_type: cmd.opportunity_type,
            workplace_type: cmd.workplace_type,
            remote_scope: cmd.remote_scope,
            experience_level: cmd.experience_level,
            salary: cmd.salary,
            is_urgent: cmd.is_urgent.unwrap_or(false),
            working_hours: cmd.working_hours,
            gender_preference: cmd.gender_preference.unwrap_or_else(|| "any".to_string()),
            has_insurance: cmd.has_insurance.unwrap_or(false),
            location_ids: cmd.location_ids,
            skill_ids: cmd.skill_ids,
        };

        new_opp.validate()?;
        let created = self.opp_repo.create_opportunity(&new_opp).await?;
        Ok(created)
    }

    /// اکشن انتشار آگهی: کسر خودکار هزینه انتشار (تعرفه srv_ad_standard) از کیف پول کارفرما
    pub async fn publish(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        // استعلام تعرفه انتشار آگهی استاندارد
        let tariff = self.finance_repo.find_tariff_by_code("srv_ad_standard").await?
            .ok_or_else(|| ApplicationError::Validation("تعرفه ثبت آگهی یافت نشد".into()))?;

        // کسر اتمیک و امن از کیف پول (در صورت کمبود موجودی خطای شفاف برمی‌گردد)
        self.finance_repo.deduct_balance(
            opp.company_id,
            tariff.price,
            "ad_publish",
            Some(opp_id),
            &format!("هزینه انتشار ۳۰ روزه آگهی «{}»", &opp.title),
        ).await?;

        let next_status = opp.status.transition_to(OpportunityStatus::Published)?;
        let now = Utc::now();
        let expires_at = now + Duration::days(30);

        self.opp_repo.update_status(opp_id, next_status, Some(now), Some(expires_at)).await?;
        Ok(())
    }

    /// نردبان آگهی روی نقشه: کسر ۵۰ هزار تومان و انتقال فوری به صدر نتایج
    pub async fn ladder(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let tariff = self.finance_repo.find_tariff_by_code("srv_ad_ladder").await?
            .ok_or_else(|| ApplicationError::Validation("تعرفه نردبان یافت نشد".into()))?;

        self.finance_repo.deduct_balance(
            opp.company_id,
            tariff.price,
            "ad_ladder",
            Some(opp_id),
            &format!("نردبان آگهی «{}» روی نقشه", &opp.title),
        ).await?;

        self.opp_repo.ladder(opp_id).await?;
        Ok(())
    }

    /// ارتقا به سنجاق طلایی روی نقشه: کسر ۱۵۰ هزار تومان و نمایش برجسته در تمام زوم‌ها
    pub async fn feature_pin(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let tariff = self.finance_repo.find_tariff_by_code("srv_featured_pin").await?
            .ok_or_else(|| ApplicationError::Validation("تعرفه سنجاق طلایی یافت نشد".into()))?;

        self.finance_repo.deduct_balance(
            opp.company_id,
            tariff.price,
            "featured_pin",
            Some(opp_id),
            &format!("سنجاق طلایی ۷ روزه برای آگهی «{}»", &opp.title),
        ).await?;

        self.opp_repo.set_featured(opp_id, true).await?;
        Ok(())
    }

    pub async fn pause(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let next_status = opp.status.transition_to(OpportunityStatus::Paused)?;
        self.opp_repo.update_status(opp_id, next_status, None, None).await?;
        Ok(())
    }

    pub async fn resume(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let next_status = opp.status.transition_to(OpportunityStatus::Published)?;
        self.opp_repo.update_status(opp_id, next_status, None, None).await?;
        Ok(())
    }

    pub async fn close(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let next_status = opp.status.transition_to(OpportunityStatus::Closed)?;
        self.opp_repo.update_status(opp_id, next_status, None, None).await?;
        Ok(())
    }

    async fn authorize_company_actor(&self, user_id: Uuid, company_id: Uuid) -> Result<(), ApplicationError> {
        let role = self
            .company_repo
            .get_user_role(company_id, user_id)
            .await?
            .ok_or_else(|| ApplicationError::Unauthorized("شما دسترسی مدیریت آگهی‌های این کسب‌وکار را ندارید".into()))?;

        if !role.can_manage_opportunities() {
            return Err(ApplicationError::Unauthorized(
                "دسترسی شما کافی نیست".into(),
            ));
        }
        Ok(())
    }
}