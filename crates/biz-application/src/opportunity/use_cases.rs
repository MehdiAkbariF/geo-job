use super::dto::CreateOpportunityCommand;
use crate::error::ApplicationError;
use biz_domain::company::CompanyRole;
use biz_domain::opportunity::{NewOpportunity, Opportunity, OpportunityStatus};
use biz_storage::{CompanyRepository, OpportunityRepository, StorageError};
use chrono::{Duration, Utc};
use uuid::Uuid;


#[derive(Clone)]
pub struct OpportunityUseCases {
    opp_repo: OpportunityRepository,
    company_repo: CompanyRepository,
}

impl OpportunityUseCases {
    pub fn new(opp_repo: OpportunityRepository, company_repo: CompanyRepository) -> Self {
        Self { opp_repo, company_repo }
    }

    /// Creates an opportunity in DRAFT state after verifying employer authorization
    pub async fn create_opportunity(
        &self,
        actor_user_id: Uuid,
        cmd: CreateOpportunityCommand,
    ) -> Result<Opportunity, ApplicationError> {
        let role = self
            .company_repo
            .get_user_role(cmd.company_id, actor_user_id)
            .await?
            .ok_or_else(|| ApplicationError::Unauthorized("User is not a member of this company".into()))?;

        if !role.can_manage_opportunities() {
            return Err(ApplicationError::Unauthorized(
                "Role does not have permission to create opportunities".into(),
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
            location_ids: cmd.location_ids,
            skill_ids: cmd.skill_ids,
        };

        new_opp.validate()?;
        let created = self.opp_repo.create_opportunity(&new_opp).await?;
        Ok(created)
    }

    /// Action: POST /opportunities/{id}/publish (DRAFT -> PUBLISHED)
    pub async fn publish(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let next_status = opp.status.transition_to(OpportunityStatus::Published)?;
        let now = Utc::now();
        let expires_at = now + Duration::days(30); // 30 days default discovery expiration

        self.opp_repo.update_status(opp_id, next_status, Some(now), Some(expires_at)).await?;
        Ok(())
    }

    /// Action: POST /opportunities/{id}/pause (PUBLISHED -> PAUSED)
    pub async fn pause(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let next_status = opp.status.transition_to(OpportunityStatus::Paused)?;
        self.opp_repo.update_status(opp_id, next_status, None, None).await?;
        Ok(())
    }

    /// Action: POST /opportunities/{id}/resume (PAUSED -> PUBLISHED)
    pub async fn resume(&self, actor_user_id: Uuid, opp_id: Uuid) -> Result<(), ApplicationError> {
        let opp = self.opp_repo.find_by_id(opp_id).await?.ok_or(StorageError::UserNotFound)?;
        self.authorize_company_actor(actor_user_id, opp.company_id).await?;

        let next_status = opp.status.transition_to(OpportunityStatus::Published)?;
        self.opp_repo.update_status(opp_id, next_status, None, None).await?;
        Ok(())
    }

    /// Action: POST /opportunities/{id}/close (PUBLISHED/PAUSED -> CLOSED) [TERMINAL]
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
            .ok_or_else(|| ApplicationError::Unauthorized("Not a member of this company".into()))?;

        if !role.can_manage_opportunities() {
            return Err(ApplicationError::Unauthorized(
                "Insufficient permissions to manage opportunity lifecycle".into(),
            ));
        }
        Ok(())
    }
}