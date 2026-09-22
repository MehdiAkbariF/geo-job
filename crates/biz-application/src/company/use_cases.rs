use super::dto::{
    AddCompanyLocationCommand, AddMemberCommand, CompanyDto, CompanyLocationDto,
    CompanyMemberDto, CompanyPublicOpportunityDto, CompanySummaryDto, CreateCompanyCommand,
    OnboardCompanyCommand, UpdateCompanyCommand, UserCompanyMembershipDto,
};
use crate::error::ApplicationError;
use biz_domain::company::{CompanyRole, NewCompany};
use biz_domain::opportunity::Opportunity;
use biz_storage::{CompanyRepository, GovernanceRepository, OpportunityRepository, StorageError};
use uuid::Uuid;

#[derive(Clone)]
pub struct CompanyUseCases {
    company_repo: CompanyRepository,
    opp_repo: OpportunityRepository,
    gov_repo: GovernanceRepository,
}

impl CompanyUseCases {
    pub fn new(
        company_repo: CompanyRepository,
        opp_repo: OpportunityRepository,
        gov_repo: GovernanceRepository,
    ) -> Self {
        Self {
            company_repo,
            opp_repo,
            gov_repo,
        }
    }

    /// Onboards an employer atomically (Creates Company + Owner membership + Submits Verification Dossier)
    pub async fn onboard_company(
        &self,
        user_id: Uuid,
        cmd: OnboardCompanyCommand,
    ) -> Result<CompanyDto, ApplicationError> {
        let new_comp = NewCompany {
            name: cmd.name,
            slug: cmd.slug,
            description: cmd.description,
            website: cmd.website,
        };

        // ۱. ایجاد شرکت و عضویت Owner
        let (created, _) = self.company_repo.create_company(user_id, &new_comp).await?;

        // ۲. ارسال فوری پرونده احراز هویت قانونی در صف ادمین
        let mut evidence = vec![
            format!("reg_no:{}", cmd.registration_number.trim()),
            format!("national_id:{}", cmd.national_id.trim()),
        ];
        if let Some(license_key) = cmd.license_storage_key {
            evidence.push(format!("license:{}", license_key));
        }

        let _ = self.gov_repo.submit_verification(created.id, &evidence).await?;

        // ۳. ثبت لاگ حسابرسی
        let _ = self.gov_repo.append_audit_log(
            Some(user_id),
            "employer_onboard",
            "company",
            created.id,
            &serde_json::json!({ "name": created.name, "reg_no": cmd.registration_number }),
        ).await;

        Ok(CompanyDto {
            id: created.id,
            name: created.name,
            slug: created.slug,
            description: created.description,
            logo_storage_key: created.logo_storage_key,
            website: created.website,
            verification_status: "pending".to_string(),
        })
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

    /// Lists all companies where the current authenticated user has an active membership role
    pub async fn list_my_companies(&self, user_id: Uuid) -> Result<Vec<UserCompanyMembershipDto>, ApplicationError> {
        let rows = self.company_repo.list_user_companies(user_id).await?;
        Ok(rows.into_iter().map(|(comp, role)| UserCompanyMembershipDto {
            company: CompanyDto {
                id: comp.id,
                name: comp.name,
                slug: comp.slug,
                description: comp.description,
                logo_storage_key: comp.logo_storage_key,
                website: comp.website,
                verification_status: format!("{:?}", comp.verification_status).to_lowercase(),
            },
            role: role.as_str().to_string(),
        }).collect())
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

    pub async fn list_locations(&self, company_id: Uuid) -> Result<Vec<CompanyLocationDto>, ApplicationError> {
        let rows = self.company_repo.list_company_locations(company_id).await?;
        Ok(rows.into_iter().map(|r| CompanyLocationDto {
            location_id: r.location_id,
            address_summary: r.address_summary,
            coordinates: [r.longitude, r.latitude],
            is_headquarters: r.is_headquarters,
        }).collect())
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

    /// واکشی آگهی‌های عمومی منتشر شده شرکت همراه با مختصات و آدرس دقیق شعبه در نقشه
    pub async fn list_public_opportunities(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<CompanyPublicOpportunityDto>, ApplicationError> {
        let rows = self.opp_repo.list_public_with_locations(company_id).await?;

        Ok(rows.into_iter().map(|r| {
            let coords = match (r.longitude, r.latitude) {
                (Some(lon), Some(lat)) => Some([lon, lat]),
                _ => None,
            };

            let summary = if r.description.chars().count() > 180 {
                format!("{}...", r.description.chars().take(180).collect::<String>())
            } else {
                r.description.clone()
            };

            CompanyPublicOpportunityDto {
                id: r.id,
                title: r.title,
                description: r.description,
                description_summary: summary,
                category_id: r.category_id,
                occupation_id: r.occupation_id,
                opportunity_type: r.opportunity_type,
                workplace_type: r.workplace_type,
                remote_scope: r.remote_scope,
                experience_level: r.experience_level,
                salary_min: r.salary_min,
                salary_max: r.salary_max,
                salary_currency: r.salary_currency,
                salary_period: r.salary_period,
                status: r.status,
                published_at: r.published_at,
                expires_at: r.expires_at,
                company: CompanySummaryDto {
                    id: r.company_id,
                    name: r.company_name,
                    slug: r.company_slug,
                    logo_storage_key: r.company_logo,
                },
                location_id: r.location_id,
                location_summary: r.location_summary,
                coordinates: coords,
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        }).collect())
    }
}