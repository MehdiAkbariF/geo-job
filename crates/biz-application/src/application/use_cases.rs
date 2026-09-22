use super::dto::{
    ApplicantSummaryDto, ApplicationDossierDto, ApplicationDto, ChangeApplicationStatusCommand,
    SubmitApplicationCommand,
};
use crate::candidate::CandidateUseCases;
use crate::error::ApplicationError;
use biz_domain::application::{Application, ApplicationStatus, NewApplication};
use biz_domain::company::CompanyVerificationStatus;
use biz_domain::opportunity::OpportunityStatus;
use biz_storage::{
    ApplicationRepository, CandidateRepository, CompanyRepository, OpportunityRepository, StorageError,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct ApplicationUseCases {
    app_repo: ApplicationRepository,
    opp_repo: OpportunityRepository,
    candidate_repo: CandidateRepository,
    company_repo: CompanyRepository,
}

impl ApplicationUseCases {
    pub fn new(
        app_repo: ApplicationRepository,
        opp_repo: OpportunityRepository,
        candidate_repo: CandidateRepository,
        company_repo: CompanyRepository,
    ) -> Self {
        Self {
            app_repo,
            opp_repo,
            candidate_repo,
            company_repo,
        }
    }

    /// Submits application with Self-Application Invariant guard
    pub async fn submit_application(
        &self,
        user_id: Uuid,
        cmd: SubmitApplicationCommand,
    ) -> Result<ApplicationDto, ApplicationError> {
        let candidate = self
            .candidate_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        let opportunity = self
            .opp_repo
            .find_by_id(cmd.opportunity_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        if opportunity.status != OpportunityStatus::Published {
            return Err(ApplicationError::Validation(
                "این فرصت شغلی در حال حاضر پذیرای درخواست همکاری نیست".into(),
            ));
        }

        // خط قرمز بیزینس: منع اپلای برای آگهی‌های شرکتی که کاربر در آن عضویت دارد
        let is_company_member = self.company_repo.get_user_role(opportunity.company_id, user_id).await?.is_some();
        if is_company_member {
            return Err(ApplicationError::Validation(
                "شما عضوی از تیم این شرکت هستید و نمی‌توانید برای موقعیت‌های شغلی سازمان خود رزومه ارسال کنید.".into(),
            ));
        }

        let new_app = NewApplication {
            candidate_id: candidate.id,
            opportunity_id: opportunity.id,
            resume_id: cmd.resume_id,
            cover_letter: cmd.cover_letter,
        };

        let created = self.app_repo.submit(&new_app).await?;

        Ok(ApplicationDto {
            id: created.id,
            candidate_id: created.candidate_id,
            opportunity_id: created.opportunity_id,
            status: created.status.as_str().to_string(),
        })
    }

    pub async fn change_status(
        &self,
        actor_user_id: Uuid,
        application_id: Uuid,
        cmd: ChangeApplicationStatusCommand,
    ) -> Result<(), ApplicationError> {
        let app = self
            .app_repo
            .find_by_id(application_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        let opp = self
            .opp_repo
            .find_by_id(app.opportunity_id)
            .await?
            .ok_or(StorageError::UserNotFound)?;

        let role = self
            .company_repo
            .get_user_role(opp.company_id, actor_user_id)
            .await?
            .ok_or_else(|| ApplicationError::Unauthorized("شما دسترسی لازم برای این شرکت را ندارید".into()))?;

        if !role.can_review_applications() {
            return Err(ApplicationError::Unauthorized(
                "نقش شما اجازه تغییر مرحله استخدامی را ندارد".into(),
            ));
        }

        let next_status = ApplicationStatus::from_str(&cmd.status)
            .ok_or_else(|| ApplicationError::Validation("مقدار وضعیت نامعتبر است".into()))?;

        let validated_status = app.status.transition_to(next_status)?;
        self.app_repo.update_status(application_id, validated_status).await?;

        Ok(())
    }

    pub async fn list_by_opportunity(
        &self,
        actor_user_id: Uuid,
        opportunity_id: Uuid,
    ) -> Result<Vec<ApplicantSummaryDto>, ApplicationError> {
        let opp = self.opp_repo.find_by_id(opportunity_id).await?.ok_or(StorageError::UserNotFound)?;
        let role = self.company_repo.get_user_role(opp.company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("شما عضو این شرکت نیستید".into())
        })?;

        if !role.can_review_applications() {
            return Err(ApplicationError::Unauthorized("عدم دسترسی کافی".into()));
        }

        let apps = self.app_repo.list_by_opportunity(opportunity_id).await?;
        let mut summaries = Vec::with_capacity(apps.len());

        for a in apps {
            let (cand_name, headline, city) = match self.candidate_repo.find_by_id(a.candidate_id).await? {
                Some(c) => (format!("{} {}", c.first_name, c.last_name), c.headline, c.preferred_city),
                None => ("کارجو".to_string(), None, None),
            };

            let resume_filename = if let Some(res_id) = a.resume_id {
                let resumes = self.candidate_repo.list_resumes(a.candidate_id).await.unwrap_or_default();
                resumes.into_iter().find(|r| r.id == res_id).map(|r| r.filename)
            } else {
                None
            };

            summaries.push(ApplicantSummaryDto {
                application_id: a.id,
                candidate_id: a.candidate_id,
                candidate_name: cand_name,
                candidate_headline: headline,
                candidate_city: city,
                status: a.status.as_str().to_string(),
                match_score: None,
                cover_letter: a.cover_letter,
                resume_id: a.resume_id,
                resume_filename,
                created_at: a.created_at,
            });
        }

        Ok(summaries)
    }

    /// گیت امنیتی پرونده متقاضی: شرکت حتماً باید احراز هویت شده باشد
    pub async fn get_application_dossier(
        &self,
        actor_user_id: Uuid,
        application_id: Uuid,
    ) -> Result<ApplicationDossierDto, ApplicationError> {
        let app = self.app_repo.find_by_id(application_id).await?.ok_or(StorageError::UserNotFound)?;
        let opp = self.opp_repo.find_by_id(app.opportunity_id).await?.ok_or(StorageError::UserNotFound)?;

        let role = self.company_repo.get_user_role(opp.company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("شما عضو این شرکت نیستید".into())
        })?;

        if !role.can_review_applications() {
            return Err(ApplicationError::Unauthorized("عدم دسترسی برای بررسی رزومه".into()));
        }

        let company = self.company_repo.find_by_id(opp.company_id).await?.ok_or(StorageError::UserNotFound)?;
        if company.verification_status != CompanyVerificationStatus::Verified {
            return Err(ApplicationError::Unauthorized(
                "مشاهده اطلاعات پرونده کامل و شماره تماس کارجویان نیازمند احراز هویت رسمی شرکت است.".into(),
            ));
        }

        let candidate = self.candidate_repo.find_by_id(app.candidate_id).await?
            .ok_or(StorageError::UserNotFound)?;

        let cand_use_cases = CandidateUseCases::new(self.candidate_repo.clone(), self.app_repo.clone());
        let profile = cand_use_cases.get_full_profile(candidate.user_id).await?;

        let candidate_coords = self.candidate_repo.get_preferred_coordinates(candidate.user_id).await?;
        let opp_coords = self.opp_repo.get_first_location_coords(opp.id).await?;

        let commute_distance = match (candidate_coords, opp_coords) {
            (Some((c_lon, c_lat)), Some((o_lon, o_lat))) => {
                let d_lng = (c_lon - o_lon) * 111.320 * (c_lat.to_radians()).cos();
                let d_lat = (c_lat - o_lat) * 110.574;
                Some((d_lng * d_lng + d_lat * d_lat).sqrt() * 1000.0)
            }
            _ => None,
        };

        Ok(ApplicationDossierDto {
            application: app,
            candidate_profile: profile,
            match_score: Some(85),
            commute_distance_meters: commute_distance,
        })
    }
}