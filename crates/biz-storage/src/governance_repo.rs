use crate::error::StorageError;
use biz_domain::governance::{AdminRole, Report, VerificationStatus};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct CompanyVerificationDbRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub status: String,
    pub evidence_storage_keys: Vec<String>,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct ReportDbRow {
    pub id: Uuid,
    pub reporter_user_id: Option<Uuid>,
    pub opportunity_id: Uuid,
    pub reason: String,
    pub details: Option<String>,
}

#[derive(Clone)]
pub struct GovernanceRepository {
    pool: PgPool,
}

impl GovernanceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_admin_role(&self, user_id: Uuid) -> Result<Option<AdminRole>, StorageError> {
        let sql = "SELECT role FROM admin_users WHERE user_id = $1";
        let role_str: Option<String> = sqlx::query_scalar(sql).bind(user_id).fetch_optional(&self.pool).await?;
        Ok(role_str.as_deref().and_then(AdminRole::from_str))
    }

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

    pub async fn list_reports(&self) -> Result<Vec<Report>, StorageError> {
        let sql = "SELECT id, reporter_user_id, opportunity_id, reason, details FROM reports ORDER BY created_at DESC";
        let rows = sqlx::query_as::<_, ReportDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|r| Report {
                id: r.id,
                reporter_user_id: r.reporter_user_id,
                opportunity_id: r.opportunity_id,
                reason: r.reason,
                details: r.details,
            })
            .collect())
    }

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

    pub async fn list_pending_verifications(&self) -> Result<Vec<CompanyVerificationDbRow>, StorageError> {
        let sql = r#"
            SELECT id, company_id, status, evidence_storage_keys, submitted_at, reviewed_at, review_notes
            FROM company_verifications
            WHERE status = 'pending'
            ORDER BY submitted_at ASC
        "#;
        let rows = sqlx::query_as::<_, CompanyVerificationDbRow>(sql).fetch_all(&self.pool).await?;
        Ok(rows)
    }

    pub async fn review_verification(
        &self,
        verification_id: Uuid,
        reviewer_id: Uuid,
        new_status: VerificationStatus,
        notes: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Uuid, StorageError> {
        let mut tx = self.pool.begin().await?;

        let sql = r#"
            UPDATE company_verifications
            SET status = $2,
                reviewer_id = $3,
                review_notes = $4,
                reviewed_at = NOW(),
                expires_at = $5,
                updated_at = NOW()
            WHERE id = $1
            RETURNING company_id
        "#;
        let company_id: Uuid = sqlx::query_scalar(sql)
            .bind(verification_id)
            .bind(new_status.as_str())
            .bind(reviewer_id)
            .bind(notes)
            .bind(expires_at)
            .fetch_one(&mut *tx)
            .await?;

        // در صورت تأیید شدن، استاتوس شرکت در جدول اصلی نیز آپدیت می‌شود
        if new_status == VerificationStatus::Verified {
            sqlx::query("UPDATE companies SET verification_status = 'verified', updated_at = NOW() WHERE id = $1")
                .bind(company_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(company_id)
    }
}