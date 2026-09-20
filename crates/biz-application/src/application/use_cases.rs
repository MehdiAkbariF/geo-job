use super::dto::{ApplicationDto, ChangeApplicationStatusCommand, SubmitApplicationCommand};
use crate::error::ApplicationError;
use biz_domain::application::{Application, ApplicationStatus, NewApplication};
use biz_domain::opportunity::OpportunityStatus;
use biz_storage::{
    ApplicationRepository, CandidateRepository, CompanyRepository, OpportunityRepository, StorageError,
};
use uuid::Uuid;

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

    /// Candidate Workflow: Submit application to an open opportunity
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

        // Only PUBLISHED opportunities accept applications (Section 37)
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

    /// Employer Workflow: Change application status through hiring pipeline
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

        // Authorize that actor belongs to the hiring company and has review permissions
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
}