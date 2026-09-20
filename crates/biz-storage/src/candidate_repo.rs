use crate::error::StorageError;
use biz_domain::candidate::{Candidate, CandidateEducation, CandidateExperience, CandidateResume};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct CandidateDbRow {
    id: Uuid,
    user_id: Uuid,
    first_name: String,
    last_name: String,
    headline: Option<String>,
    bio: Option<String>,
    avatar_storage_key: Option<String>,
    preferred_location_id: Option<Uuid>,
    preferred_city: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl CandidateDbRow {
    fn to_domain(self) -> Candidate {
        Candidate {
            id: self.id,
            user_id: self.user_id,
            first_name: self.first_name,
            last_name: self.last_name,
            headline: self.headline,
            bio: self.bio,
            avatar_storage_key: self.avatar_storage_key,
            preferred_location_id: self.preferred_location_id,
            preferred_city: self.preferred_city,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Clone)]
pub struct CandidateRepository {
    pool: PgPool,
}

impl CandidateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Candidate>, StorageError> {
        let sql = "SELECT * FROM candidates WHERE user_id = $1";
        let row: Option<CandidateDbRow> = sqlx::query_as(sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(CandidateDbRow::to_domain))
    }

    pub async fn upsert_profile(
        &self,
        user_id: Uuid,
        first_name: &str,
        last_name: &str,
        headline: Option<&str>,
        bio: Option<&str>,
        preferred_city: Option<&str>,
    ) -> Result<Candidate, StorageError> {
        let sql = r#"
            INSERT INTO candidates (user_id, first_name, last_name, headline, bio, preferred_city)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (user_id) 
            DO UPDATE SET
                first_name = EXCLUDED.first_name,
                last_name = EXCLUDED.last_name,
                headline = EXCLUDED.headline,
                bio = EXCLUDED.bio,
                preferred_city = EXCLUDED.preferred_city,
                updated_at = NOW()
            RETURNING *
        "#;

        let row = sqlx::query_as::<_, CandidateDbRow>(sql)
            .bind(user_id)
            .bind(first_name)
            .bind(last_name)
            .bind(headline)
            .bind(bio)
            .bind(preferred_city)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.to_domain())
    }

    // Skills Management
    pub async fn set_skills(&self, candidate_id: Uuid, skill_ids: &[Uuid]) -> Result<(), StorageError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM candidate_skills WHERE candidate_id = $1")
            .bind(candidate_id)
            .execute(&mut *tx)
            .await?;

        for skill_id in skill_ids {
            sqlx::query("INSERT INTO candidate_skills (candidate_id, skill_id) VALUES ($1, $2)")
                .bind(candidate_id)
                .bind(skill_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    // Experience Management
    pub async fn add_experience(
        &self,
        candidate_id: Uuid,
        title: &str,
        company_name: &str,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        is_current: bool,
        description: Option<&str>,
    ) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO candidate_experiences (candidate_id, title, company_name, start_date, end_date, is_current, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(candidate_id)
            .bind(title)
            .bind(company_name)
            .bind(start_date)
            .bind(end_date)
            .bind(is_current)
            .bind(description)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    // Resume Management
    pub async fn add_resume(
        &self,
        candidate_id: Uuid,
        storage_key: &str,
        filename: &str,
        mime_type: &str,
        file_size: i64,
    ) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO candidate_resumes (candidate_id, storage_key, filename, mime_type, file_size)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(candidate_id)
            .bind(storage_key)
            .bind(filename)
            .bind(mime_type)
            .bind(file_size)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }
}