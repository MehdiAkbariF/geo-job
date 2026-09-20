use super::dto::{
    CompanyVerificationDto, CreateReportCommand, ReviewVerificationCommand,
    SubmitVerificationCommand,
};
use crate::error::ApplicationError;
use biz_domain::governance::{Report, VerificationStatus};
use biz_storage::{CompanyRepository, GovernanceRepository};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct GovernanceUseCases {
    gov_repo: GovernanceRepository,
    company_repo: CompanyRepository,
}

impl GovernanceUseCases {
    pub fn new(gov_repo: GovernanceRepository, company_repo: CompanyRepository) -> Self {
        Self { gov_repo, company_repo }
    }

    pub async fn report_opportunity(
        &self,
        reporter_user_id: Option<Uuid>,
        cmd: CreateReportCommand,
    ) -> Result<Uuid, ApplicationError> {
        let report_id = self
            .gov_repo
            .create_report(reporter_user_id, cmd.opportunity_id, &cmd.reason, cmd.details.as_deref())
            .await?;

        self.gov_repo
            .append_audit_log(
                reporter_user_id,
                "create_report",
                "opportunity",
                cmd.opportunity_id,
                &serde_json::json!({ "report_id": report_id, "reason": cmd.reason }),
            )
            .await?;

        Ok(report_id)
    }

    pub async fn submit_company_verification(
        &self,
        actor_user_id: Uuid,
        cmd: SubmitVerificationCommand,
    ) -> Result<Uuid, ApplicationError> {
        let role = self
            .company_repo
            .get_user_role(cmd.company_id, actor_user_id)
            .await?
            .ok_or_else(|| ApplicationError::Unauthorized("Not a member of this company".into()))?;

        if !role.can_edit_company() {
            return Err(ApplicationError::Unauthorized(
                "Only Owner or Admin can submit company verification".into(),
            ));
        }

        let ver_id = self
            .gov_repo
            .submit_verification(cmd.company_id, &cmd.evidence_storage_keys)
            .await?;

        self.gov_repo
            .append_audit_log(
                Some(actor_user_id),
                "submit_verification",
                "company",
                cmd.company_id,
                &serde_json::json!({ "verification_id": ver_id }),
            )
            .await?;

        Ok(ver_id)
    }

    pub async fn list_pending_verifications(&self, actor_user_id: Uuid) -> Result<Vec<CompanyVerificationDto>, ApplicationError> {
        self.authorize_platform_admin(actor_user_id).await?;
        let rows = self.gov_repo.list_pending_verifications().await?;

        Ok(rows
            .into_iter()
            .map(|r| CompanyVerificationDto {
                id: r.id,
                company_id: r.company_id,
                status: r.status,
                evidence_storage_keys: r.evidence_storage_keys,
                submitted_at: r.submitted_at,
                reviewed_at: r.reviewed_at,
                review_notes: r.review_notes,
            })
            .collect())
    }

    pub async fn review_verification(
        &self,
        actor_user_id: Uuid,
        verification_id: Uuid,
        cmd: ReviewVerificationCommand,
    ) -> Result<(), ApplicationError> {
        self.authorize_platform_admin(actor_user_id).await?;

        let status = VerificationStatus::from_str(&cmd.status)
            .ok_or_else(|| ApplicationError::Validation("Invalid status".into()))?;

        let expires_at = if status == VerificationStatus::Verified {
            Some(Utc::now() + Duration::days(365)) // اعتبار تاییدیه به مدت ۱ سال
        } else {
            None
        };

        let company_id = self
            .gov_repo
            .review_verification(
                verification_id,
                actor_user_id,
                status,
                cmd.notes.as_deref(),
                expires_at,
            )
            .await?;

        self.gov_repo
            .append_audit_log(
                Some(actor_user_id),
                "review_verification",
                "company",
                company_id,
                &serde_json::json!({ "verification_id": verification_id, "status": cmd.status }),
            )
            .await?;

        Ok(())
    }

    pub async fn list_reports(&self, actor_user_id: Uuid) -> Result<Vec<Report>, ApplicationError> {
        self.authorize_platform_admin(actor_user_id).await?;
        let reports = self.gov_repo.list_reports().await?;
        Ok(reports)
    }

    async fn authorize_platform_admin(&self, user_id: Uuid) -> Result<(), ApplicationError> {
        let role = self.gov_repo.get_admin_role(user_id).await?;
        match role {
            Some(r) if r.can_verify_companies() => Ok(()),
            _ => Err(ApplicationError::Unauthorized("Requires platform admin role".into())),
        }
    }
}