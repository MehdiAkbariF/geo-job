use super::dto::{CreateReportCommand, SubmitVerificationCommand};
use crate::error::ApplicationError;
use biz_storage::{CompanyRepository, GovernanceRepository};
use uuid::Uuid;

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

        // Append immutable audit log entry (Section 47)
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
}