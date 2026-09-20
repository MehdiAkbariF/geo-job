use crate::error::StorageError;
use biz_domain::governance::{AdminRole, VerificationStatus};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct GovernanceRepository {
    pool: PgPool,
}

impl GovernanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // --- Platform Admin Authorization ---
    pub async fn get_admin_role(&self, user_id: Uuid) -> Result<Option<AdminRole>, StorageError> {
        let sql = "SELECT role FROM admin_users WHERE user_id = $1";
        let role_str: Option<String> = sqlx::query_scalar(sql).bind(user_id).fetch_optional(&self.pool).await?;
        Ok(role_str.as_deref().and_then(AdminRole::from_str))
    }

    // --- Immutable Append-Only Audit Log ---
    pub async fn append_audit_log(
        &self,
        actor_id: Option<Uuid>,
        action: &str,
        resource_type: &str,
        resource_id: Uuid,
        metadata: &serde_json::Value,
    ) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO audit_logs (actor_id, action, resource_type, resource_id, metadata)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
        "#;
        let id: Uuid = sqlx::query_scalar(sql)
            .bind(actor_id)
            .bind(action)
            .bind(resource_type)
            .bind(resource_id)
            .bind(metadata)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    // --- Reports (Allegation) ---
    pub async fn create_report(
        &self,
        reporter_id: Option<Uuid>,
        opportunity_id: Uuid,
        reason: &str,
        details: Option<&str>,
    ) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO reports (reporter_user_id, opportunity_id, reason, details)
            VALUES ($1, $2, $3, $4)
            RETURNING id
        "#;
        let id: Uuid = sqlx::query_scalar(sql)
            .bind(reporter_id)
            .bind(opportunity_id)
            .bind(reason)
            .bind(details)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    // --- Company Verification Submission ---
    pub async fn submit_verification(
        &self,
        company_id: Uuid,
        evidence_keys: &[String],
    ) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO company_verifications (company_id, status, evidence_storage_keys, submitted_at)
            VALUES ($1, 'pending', $2, NOW())
            RETURNING id
        "#;
        let id: Uuid = sqlx::query_scalar(sql)
            .bind(company_id)
            .bind(evidence_keys)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    // --- Moderator Review Action ---
    pub async fn review_verification(
        &self,
        verification_id: Uuid,
        reviewer_id: Uuid,
        new_status: VerificationStatus,
        notes: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), StorageError> {
        let sql = r#"
            UPDATE company_verifications
            SET status = $2,
                reviewer_id = $3,
                review_notes = $4,
                reviewed_at = NOW(),
                expires_at = $5,
                updated_at = NOW()
            WHERE id = $1
        "#;
        sqlx::query(sql)
            .bind(verification_id)
            .bind(new_status.as_str())
            .bind(reviewer_id)
            .bind(notes)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}