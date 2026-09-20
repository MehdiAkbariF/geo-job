use crate::error::StorageError;
use biz_domain::application::{Application, ApplicationStatus, NewApplication};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct ApplicationDbRow {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub opportunity_id: Uuid,
    pub resume_id: Option<Uuid>,
    pub cover_letter: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ApplicationDbRow {
    fn to_domain(self) -> Result<Application, StorageError> {
        let status = ApplicationStatus::from_str(&self.status)
            .ok_or_else(|| StorageError::Security("Corrupt application status".into()))?;

        Ok(Application {
            id: self.id,
            candidate_id: self.candidate_id,
            opportunity_id: self.opportunity_id,
            resume_id: self.resume_id,
            cover_letter: self.cover_letter,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct ApplicationRepository {
    pool: PgPool,
}

impl ApplicationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Submits an application and enforces database unique constraint against duplicates (Section 22)
    pub async fn submit(&self, item: &NewApplication) -> Result<Application, StorageError> {
        let sql = r#"
            INSERT INTO applications (candidate_id, opportunity_id, resume_id, cover_letter, status)
            VALUES ($1, $2, $3, $4, 'submitted')
            RETURNING *
        "#;

        let res = sqlx::query_as::<_, ApplicationDbRow>(sql)
            .bind(item.candidate_id)
            .bind(item.opportunity_id)
            .bind(item.resume_id)
            .bind(&item.cover_letter)
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(row) => row.to_domain(),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(StorageError::DuplicateApplication)
            }
            Err(e) => Err(StorageError::Database(e)),
        }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Application>, StorageError> {
        let sql = "SELECT * FROM applications WHERE id = $1";
        let row: Option<ApplicationDbRow> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(|r| r.to_domain()).transpose()
    }

    pub async fn update_status(&self, id: Uuid, new_status: ApplicationStatus) -> Result<(), StorageError> {
        let sql = "UPDATE applications SET status = $2, updated_at = NOW() WHERE id = $1";
        sqlx::query(sql)
            .bind(id)
            .bind(new_status.as_str())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn list_by_opportunity(&self, opportunity_id: Uuid) -> Result<Vec<Application>, StorageError> {
        let sql = "SELECT * FROM applications WHERE opportunity_id = $1 ORDER BY created_at DESC";
        let rows = sqlx::query_as::<_, ApplicationDbRow>(sql)
            .bind(opportunity_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(|r| r.to_domain()).collect()
    }

    pub async fn list_by_candidate(&self, candidate_id: Uuid) -> Result<Vec<Application>, StorageError> {
        let sql = "SELECT * FROM applications WHERE candidate_id = $1 ORDER BY created_at DESC";
        let rows = sqlx::query_as::<_, ApplicationDbRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter().map(|r| r.to_domain()).collect()
    }
}