use crate::error::StorageError;
use biz_domain::opportunity::{
    NewOpportunity, Opportunity, OpportunityStatus, OpportunityType, RemoteScope, Salary, WorkplaceType,
};
use biz_domain::taxonomy::ExperienceLevel;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct OpportunityDbRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub title: String,
    pub description: String,
    pub category_id: Uuid,
    pub occupation_id: Option<Uuid>,
    pub opportunity_type: String,
    pub workplace_type: String,
    pub remote_scope: Option<String>,
    pub experience_level: String,
    pub salary_min: Option<Decimal>,
    pub salary_max: Option<Decimal>,
    pub salary_currency: String,
    pub salary_period: String,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OpportunityDbRow {
    fn to_domain(self) -> Result<Opportunity, StorageError> {
        let opp_type = OpportunityType::from_str(&self.opportunity_type)
            .ok_or_else(|| StorageError::Security("Corrupt opportunity type".into()))?;
        let workplace = WorkplaceType::from_str(&self.workplace_type)
            .ok_or_else(|| StorageError::Security("Corrupt workplace type".into()))?;
        let remote = self.remote_scope.as_deref().and_then(RemoteScope::from_str);
        let exp_level = ExperienceLevel::from_str(&self.experience_level)
            .ok_or_else(|| StorageError::Security("Corrupt experience level".into()))?;
        let status = OpportunityStatus::from_str(&self.status)
            .ok_or_else(|| StorageError::Security("Corrupt status".into()))?;

        Ok(Opportunity {
            id: self.id,
            company_id: self.company_id,
            title: self.title,
            description: self.description,
            category_id: self.category_id,
            occupation_id: self.occupation_id,
            opportunity_type: opp_type,
            workplace_type: workplace,
            remote_scope: remote,
            experience_level: exp_level,
            salary: Salary {
                min: self.salary_min,
                max: self.salary_max,
                currency: self.salary_currency,
                period: self.salary_period,
            },
            status,
            published_at: self.published_at,
            expires_at: self.expires_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Clone)]
pub struct OpportunityRepository {
    pool: PgPool,
}

impl OpportunityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Atomically creates an opportunity and links its physical locations and required skills
    pub async fn create_opportunity(&self, item: &NewOpportunity) -> Result<Opportunity, StorageError> {
        let mut tx = self.pool.begin().await?;

        let sql = r#"
            INSERT INTO opportunities (
                company_id, title, description, category_id, occupation_id,
                opportunity_type, workplace_type, remote_scope, experience_level,
                salary_min, salary_max, salary_currency, salary_period, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'draft')
            RETURNING *
        "#;

        let remote_str = item.remote_scope.map(|r| r.as_str());

        let row = sqlx::query_as::<_, OpportunityDbRow>(sql)
            .bind(item.company_id)
            .bind(&item.title)
            .bind(&item.description)
            .bind(item.category_id)
            .bind(item.occupation_id)
            .bind(item.opportunity_type.as_str())
            .bind(item.workplace_type.as_str())
            .bind(remote_str)
            .bind(item.experience_level.as_str())
            .bind(item.salary.min)
            .bind(item.salary.max)
            .bind(&item.salary.currency)
            .bind(&item.salary.period)
            .fetch_one(&mut *tx)
            .await?;

        // Link multiple physical locations to PostGIS shared table
        for loc_id in &item.location_ids {
            sqlx::query("INSERT INTO opportunity_locations (opportunity_id, location_id) VALUES ($1, $2)")
                .bind(row.id)
                .bind(loc_id)
                .execute(&mut *tx)
                .await?;
        }

        // Link skills
        for skill_id in &item.skill_ids {
            sqlx::query("INSERT INTO opportunity_skills (opportunity_id, skill_id, is_required) VALUES ($1, $2, true)")
                .bind(row.id)
                .bind(skill_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        row.to_domain()
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Opportunity>, StorageError> {
        let sql = "SELECT * FROM opportunities WHERE id = $1";
        let row: Option<OpportunityDbRow> = sqlx::query_as(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(|r| r.to_domain()).transpose()
    }

    /// Updates opportunity lifecycle status atomically
    pub async fn update_status(
        &self,
        id: Uuid,
        new_status: OpportunityStatus,
        published_at: Option<DateTime<Utc>>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(), StorageError> {
        let sql = r#"
            UPDATE opportunities
            SET status = $2,
                published_at = COALESCE($3, published_at),
                expires_at = COALESCE($4, expires_at)
            WHERE id = $1
        "#;

        sqlx::query(sql)
            .bind(id)
            .bind(new_status.as_str())
            .bind(published_at)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}