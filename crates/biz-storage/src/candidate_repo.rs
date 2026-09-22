use crate::error::StorageError;
use biz_domain::candidate::{Candidate, CandidateEducation, CandidateExperience, CandidateResume};
use biz_domain::saved::CandidatePreferences;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
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

#[derive(Debug, sqlx::FromRow)]
struct ExperienceDbRow {
    id: Uuid,
    candidate_id: Uuid,
    title: String,
    company_name: String,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
    is_current: bool,
    description: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CandidateMatchContext {
    pub candidate_id: Uuid,
    pub skill_ids: Vec<Uuid>,
    pub preferred_city: Option<String>,
    pub preferred_workplace_types: Vec<String>,
    pub expected_salary_min: Option<Decimal>,
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

    pub async fn get_preferred_coordinates(&self, user_id: Uuid) -> Result<Option<(f64, f64)>, StorageError> {
        let loc_sql = r#"
            SELECT 
                ST_X(loc.coordinates::geometry) AS longitude,
                ST_Y(loc.coordinates::geometry) AS latitude
            FROM candidates c
            INNER JOIN locations loc ON loc.id = c.preferred_location_id
            WHERE c.user_id = $1
            LIMIT 1
        "#;

        #[derive(sqlx::FromRow)]
        struct CoordsRow {
            longitude: f64,
            latitude: f64,
        }

        let row: Option<CoordsRow> = sqlx::query_as(loc_sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            return Ok(Some((r.longitude, r.latitude)));
        }

        Ok(None)
    }

    // واکشی مهارت‌ها به همراه نام برای نمایش کامل در فرانت‌اند
    pub async fn get_skills_with_names(&self, candidate_id: Uuid) -> Result<Vec<(Uuid, String)>, StorageError> {
        let sql = r#"
            SELECT s.id, s.name 
            FROM candidate_skills cs
            INNER JOIN skills s ON s.id = cs.skill_id
            WHERE cs.candidate_id = $1
            ORDER BY s.name ASC
        "#;

        #[derive(sqlx::FromRow)]
        struct SkillRow {
            id: Uuid,
            name: String,
        }

        let rows = sqlx::query_as::<_, SkillRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| (r.id, r.name)).collect())
    }

    // استخراج کانتکست تطبیق هوشمند رزومه برای موتور سازگاری PostGIS
    pub async fn get_match_context(&self, user_id: Uuid) -> Result<Option<CandidateMatchContext>, StorageError> {
        let candidate = match self.find_by_user_id(user_id).await? {
            Some(c) => c,
            None => return Ok(None),
        };

        let skills: Vec<Uuid> = sqlx::query_scalar("SELECT skill_id FROM candidate_skills WHERE candidate_id = $1")
            .bind(candidate.id)
            .fetch_all(&self.pool)
            .await?;

        let prefs = self.get_preferences(candidate.id).await?;

        let (workplaces, salary_min) = match prefs {
            Some(p) => (p.preferred_workplace_types, p.expected_salary_min),
            None => (Vec::new(), None),
        };

        Ok(Some(CandidateMatchContext {
            candidate_id: candidate.id,
            skill_ids: skills,
            preferred_city: candidate.preferred_city,
            preferred_workplace_types: workplaces,
            expected_salary_min: salary_min,
        }))
    }

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

    pub async fn delete_experience(&self, candidate_id: Uuid, exp_id: Uuid) -> Result<(), StorageError> {
        let sql = "DELETE FROM candidate_experiences WHERE id = $1 AND candidate_id = $2";
        sqlx::query(sql)
            .bind(exp_id)
            .bind(candidate_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_experiences(&self, candidate_id: Uuid) -> Result<Vec<CandidateExperience>, StorageError> {
        let sql = "SELECT * FROM candidate_experiences WHERE candidate_id = $1 ORDER BY start_date DESC";
        let rows = sqlx::query_as::<_, ExperienceDbRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| CandidateExperience {
                id: r.id,
                candidate_id: r.candidate_id,
                title: r.title,
                company_name: r.company_name,
                start_date: r.start_date,
                end_date: r.end_date,
                is_current: r.is_current,
                description: r.description,
                created_at: r.created_at,
            })
            .collect())
    }

    pub async fn get_preferences(&self, candidate_id: Uuid) -> Result<Option<CandidatePreferences>, StorageError> {
        #[derive(sqlx::FromRow)]
        struct PrefsDbRow {
            candidate_id: Uuid,
            preferred_workplace_types: Vec<String>,
            preferred_opportunity_types: Vec<String>,
            expected_salary_min: Option<Decimal>,
            salary_currency: String,
            remote_only: bool,
            updated_at: DateTime<Utc>,
        }

        let sql = "SELECT * FROM candidate_preferences WHERE candidate_id = $1";
        let row: Option<PrefsDbRow> = sqlx::query_as(sql)
            .bind(candidate_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| CandidatePreferences {
            candidate_id: r.candidate_id,
            preferred_workplace_types: r.preferred_workplace_types,
            preferred_opportunity_types: r.preferred_opportunity_types,
            expected_salary_min: r.expected_salary_min,
            salary_currency: r.salary_currency,
            remote_only: r.remote_only,
            updated_at: r.updated_at,
        }))
    }

    pub async fn upsert_preferences(
        &self,
        candidate_id: Uuid,
        workplace_types: &[String],
        opportunity_types: &[String],
        expected_salary_min: Option<Decimal>,
        salary_currency: &str,
        remote_only: bool,
    ) -> Result<(), StorageError> {
        let sql = r#"
            INSERT INTO candidate_preferences (
                candidate_id, preferred_workplace_types, preferred_opportunity_types,
                expected_salary_min, salary_currency, remote_only, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            ON CONFLICT (candidate_id) DO UPDATE SET
                preferred_workplace_types = EXCLUDED.preferred_workplace_types,
                preferred_opportunity_types = EXCLUDED.preferred_opportunity_types,
                expected_salary_min = EXCLUDED.expected_salary_min,
                salary_currency = EXCLUDED.salary_currency,
                remote_only = EXCLUDED.remote_only,
                updated_at = NOW()
        "#;

        sqlx::query(sql)
            .bind(candidate_id)
            .bind(workplace_types)
            .bind(opportunity_types)
            .bind(expected_salary_min)
            .bind(salary_currency)
            .bind(remote_only)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}