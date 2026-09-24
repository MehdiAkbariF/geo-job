use super::dto::CreateOpportunityCommand;
use crate::error::ApplicationError;
use biz_domain::company::CompanyVerificationStatus;
use biz_domain::opportunity::{NewOpportunity, Opportunity, OpportunityStatus, WorkplaceType};
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

    /// ایجاد پیش‌نویس با اعمال گیت‌های امنیتی نوع کسب‌وکار
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

        let company = self.company_repo.find_by_id(cmd.company_id).await?
            .ok_or(StorageError::UserNotFound)?;

        // 🛡️ گیت امنیتی ۱: کارفرمای پروژه‌ای هرگز حق ثبت آگهی حضوری ندارد
        if company.business_type == "individual_client" {
            if cmd.workplace_type != WorkplaceType::Remote || !cmd.is_urgent.unwrap_or(false) && cmd.location_ids.len() > 0 {
                return Err(ApplicationError::Validation(
                    "کارفرمایان پروژه‌ای تنها مجاز به ثبت درخواست‌های پروژه و دورکاری هستند. برای استخدام حضوری، باید کسب‌وکار صنفی یا شرکتی خود را به همراه پروانه کسب ثبت نمایید.".into()
                ));
            }
        }

        let is_proj = company.business_type == "individual_client" || cmd.workplace_type == WorkplaceType::Remote;

        let new_opp = NewOpportunity {
            company_id: cmd.company_id,
            title: cmd.title,
            description: cmd.description,
            category_id: cmd.category_id,
            occupation_id: cmd.occupation_id,
            opportunity_type: cmd.opportunity_type,
            workplace_type: if company.business_type == "individual_client" { WorkplaceType::Remote } else { cmd.workplace_type },
            remote_scope: cmd.remote_scope,
            experience_level: cmd.experience_level,
            salary: cmd.salary,
            is_urgent: cmd.is_urgent.unwrap_or(false),
            working_hours: cmd.working_hours,
            gender_preference: cmd.gender_preference.unwrap_or_else(|| "any".to_string()),
            has_insurance: cmd.has_insurance.unwrap_or(false),
            location_ids: if company.business_type == "individual_client" { vec![] } else { cmd.location_ids },
            skill_ids: cmd.skill_ids,
        };

        new_opp.validate()?;
        let created = self.opp_repo.create_opportunity(&new_opp).await?;
        Ok(created)
    }

    /// اکشن انتشار آگهی: کسر خودکار هزینه با بررسی تکمیل مدارک قانونی کارفرما
    pub async fn publish(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let company = self.company_repo.find_by_id(opp.company_id).await?.ok_or(StorageError::UserNotFound)?;

        // 🛡️ گیت امنیتی ۲: شرکت‌ها و اصناف حضوری حتماً باید حداقل مدارک پروانه/ثبت را ارسال کرده باشند
        if company.business_type != "individual_client" && company.verification_status == CompanyVerificationStatus::NotStarted {
            return Err(ApplicationError::Validation(
                "برای انتشار آگهی استخدام حضوری، ابتدا باید مدارک پروانه کسب یا روزنامه رسمی شرکت را در بخش «ارسال مدارک» بارگذاری نمایید.".into()
            ));
        }

        // استعلام تعرفه انتشار آگهی استاندارد
        let tariff = self.finance_repo.find_tariff_by_code("srv_ad_standard").await?
            .ok_or_else(|| ApplicationError::Validation("تعرفه ثبت آگهی یافت نشد".into()))?;

        // کسر اتمیک و امن از موجودی
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