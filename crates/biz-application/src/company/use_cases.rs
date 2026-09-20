use super::dto::{
    AddCompanyLocationCommand, AddMemberCommand, CompanyDto, CompanyMemberDto,
    CreateCompanyCommand, UpdateCompanyCommand,
};
use crate::error::ApplicationError;
use biz_domain::company::{CompanyRole, NewCompany};
use biz_domain::opportunity::Opportunity;
use biz_storage::{CompanyRepository, OpportunityRepository, StorageError};
use uuid::Uuid;

#[derive(Clone)]
pub struct CompanyUseCases {
    company_repo: CompanyRepository,
    opp_repo: OpportunityRepository,
}

impl CompanyUseCases {
    pub fn new(company_repo: CompanyRepository, opp_repo: OpportunityRepository) -> Self {
        Self {
            company_repo,
            opp_repo,
        }
    }

    pub async fn create_company(&self, user_id: Uuid, cmd: CreateCompanyCommand) -> Result<CompanyDto, ApplicationError> {
        let new_comp = NewCompany {
            name: cmd.name,
            slug: cmd.slug,
            description: cmd.description,
            website: cmd.website,
        };

        let (created, _) = self.company_repo.create_company(user_id, &new_comp).await?;
        Ok(CompanyDto {
            id: created.id,
            name: created.name,
            slug: created.slug,
            description: created.description,
            logo_storage_key: created.logo_storage_key,
            website: created.website,
            verification_status: format!("{:?}", created.verification_status).to_lowercase(),
        })
    }

    pub async fn update_company(&self, actor_user_id: Uuid, company_id: Uuid, cmd: UpdateCompanyCommand) -> Result<CompanyDto, ApplicationError> {
        let role = self.company_repo.get_user_role(company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("You are not a member of this company".into())
        })?;

        if !role.can_edit_company() {
            return Err(ApplicationError::Unauthorized("Insufficient permissions to edit company".into()));
        }

        let updated = self.company_repo.update_company(
            company_id,
            &cmd.name,
            &cmd.slug,
            cmd.description.as_deref(),
            cmd.website.as_deref(),
            cmd.logo_storage_key.as_deref(),
        ).await?;

        Ok(CompanyDto {
            id: updated.id,
            name: updated.name,
            slug: updated.slug,
            description: updated.description,
            logo_storage_key: updated.logo_storage_key,
            website: updated.website,
            verification_status: format!("{:?}", updated.verification_status).to_lowercase(),
        })
    }

    pub async fn get_company(&self, company_id: Uuid) -> Result<CompanyDto, ApplicationError> {
        let c = self.company_repo.find_by_id(company_id).await?.ok_or(StorageError::UserNotFound)?;
        Ok(CompanyDto {
            id: c.id,
            name: c.name,
            slug: c.slug,
            description: c.description,
            logo_storage_key: c.logo_storage_key,
            website: c.website,
            verification_status: format!("{:?}", c.verification_status).to_lowercase(),
        })
    }

    pub async fn add_location(&self, actor_user_id: Uuid, company_id: Uuid, cmd: AddCompanyLocationCommand) -> Result<(), ApplicationError> {
        let role = self.company_repo.get_user_role(company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("Not authorized for this company".into())
        })?;

        if !role.can_edit_company() {
            return Err(ApplicationError::Unauthorized("Cannot add company location".into()));
        }

        self.company_repo.add_company_location(company_id, cmd.location_id, cmd.is_headquarters).await?;
        Ok(())
    }

    pub async fn list_members(&self, actor_user_id: Uuid, company_id: Uuid) -> Result<Vec<CompanyMemberDto>, ApplicationError> {
        let _ = self.company_repo.get_user_role(company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("Not authorized to view team members".into())
        })?;

        let members = self.company_repo.list_members(company_id).await?;
        Ok(members.into_iter().map(|m| CompanyMemberDto {
            id: m.id,
            user_id: m.user_id,
            role: m.role.as_str().to_string(),
        }).collect())
    }

    pub async fn add_member(&self, actor_user_id: Uuid, company_id: Uuid, cmd: AddMemberCommand) -> Result<(), ApplicationError> {
        let role = self.company_repo.get_user_role(company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("Not a member".into())
        })?;

        if !role.can_manage_members() {
            return Err(ApplicationError::Unauthorized("Only Owners and Admins can manage members".into()));
        }

        let new_role = CompanyRole::from_str(&cmd.role).ok_or_else(|| ApplicationError::Validation("Invalid role".into()))?;
        self.company_repo.add_member(company_id, cmd.user_id, new_role).await?;
        Ok(())
    }

    pub async fn list_company_opportunities(&self, actor_user_id: Uuid, company_id: Uuid) -> Result<Vec<Opportunity>, ApplicationError> {
        let _ = self.company_repo.get_user_role(company_id, actor_user_id).await?.ok_or_else(|| {
            ApplicationError::Unauthorized("Not a member of this company".into())
        })?;

        let list = self.opp_repo.list_by_company(company_id, false).await?;
        Ok(list)
    }

    pub async fn list_public_opportunities(&self, company_id: Uuid) -> Result<Vec<Opportunity>, ApplicationError> {
        let list = self.opp_repo.list_by_company(company_id, true).await?;
        Ok(list)
    }
}