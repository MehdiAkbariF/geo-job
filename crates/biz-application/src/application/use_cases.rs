use super::dto::{
    ApplicationDossierDto, ApplicationDto, ChangeApplicationStatusCommand,
    SubmitApplicationCommand,
};
use crate::candidate::CandidateUseCases;
use crate::error::ApplicationError;
use biz_domain::application::{Application, ApplicationStatus, NewApplication};
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
                "This opportunity is not currently accepting applications".into(),
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
            .ok_or_else(|| ApplicationError::Unauthorized("Not authorized for this company".into()))?;

        if !role.can_review_applications() {
            return Err(ApplicationError::Unauthorized(
                "Role does not have permission to change application status".into(),
            ));
        }

        let next_status = ApplicationStatus::from_str(&cmd.status)
            .ok_or_else(|| ApplicationError::Validation("Invalid status value".into()))?;

        let validated_status = app.status.transition_to(next_status)?;
        self.app_repo.update_status(application_id, validated_status).await?;

        Ok(())
    }

    pub async fn list_by_opportunity(
        &self,
        actor_user_id: Uuid,
        opportunity_id: Uuid,
    ) -> Result<Vec<Application>, ApplicationError> {
        let opp = self.opp_repo.find_by_id(opportunity_id).await?.ok_or(StorageError::UserNotFound)?;
        let role = self.company_repo.get_user_role(opp.company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("Not authorized for this company".into())
        })?;

        if !role.can_review_applications() {
            return Err(ApplicationError::Unauthorized("Insufficient permissions".into()));
        }

        let list = self.app_repo.list_by_opportunity(opportunity_id).await?;
        Ok(list)
    }

    pub async fn get_application_dossier(
        &self,
        actor_user_id: Uuid,
        application_id: Uuid,
    ) -> Result<ApplicationDossierDto, ApplicationError> {
        let app = self.app_repo.find_by_id(application_id).await?.ok_or(StorageError::UserNotFound)?;
        let opp = self.opp_repo.find_by_id(app.opportunity_id).await?.ok_or(StorageError::UserNotFound)?;

        let role = self.company_repo.get_user_role(opp.company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("Not authorized to view this applicant".into())
        })?;

        if !role.can_review_applications() {
            return Err(ApplicationError::Unauthorized("Insufficient permissions".into()));
        }

        // واکشی پروفایل کارجو بر اساس candidate_id
        let cand_use_cases = CandidateUseCases::new(self.candidate_repo.clone(), self.app_repo.clone());
        let candidate = self.candidate_repo.find_by_user_id(app.candidate_id).await?;
        let user_id = candidate.map(|c| c.user_id).unwrap_or(app.candidate_id);
        let profile = cand_use_cases.get_full_profile(user_id).await?;

        Ok(ApplicationDossierDto {
            application: app,
            candidate_profile: profile,
        })
    }
}