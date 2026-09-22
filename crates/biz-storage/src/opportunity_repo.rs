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

/// ساختار پایگاه داده جهت بازگرداندن آگهی همراه با مشخصات شرکت و مختصات جغرافیایی PostGIS
#[derive(Debug, sqlx::FromRow)]
pub struct PublicOpportunityRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_slug: String,
    pub company_logo: Option<String>,
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
    pub location_id: Option<Uuid>,
    pub location_summary: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct OpportunityRepository {
    pool: PgPool,
}

impl OpportunityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

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

        for loc_id in &item.location_ids {
            sqlx::query("INSERT INTO opportunity_locations (opportunity_id, location_id) VALUES ($1, $2)")
                .bind(row.id)
                .bind(loc_id)
                .execute(&mut *tx)
                .await?;
        }

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
        let row: Option<OpportunityDbRow> = sqlx::query_as(sql).bind(id).fetch_optional(&self.pool).await?;
        row.map(|r| r.to_domain()).transpose()
    }

    // واکشی مختصات اولین شعبه فیزیکی شغل برای محاسبه مسافت تردد کارجو
    pub async fn get_first_location_coords(&self, opp_id: Uuid) -> Result<Option<(f64, f64)>, StorageError> {
        let sql = r#"
            SELECT 
                ST_X(loc.coordinates::geometry) AS longitude,
                ST_Y(loc.coordinates::geometry) AS latitude
            FROM opportunity_locations ol
            INNER JOIN locations loc ON loc.id = ol.location_id
            WHERE ol.opportunity_id = $1
            LIMIT 1
        "#;

        #[derive(sqlx::FromRow)]
        struct CoordsRow {
            longitude: f64,
            latitude: f64,
        }

        let row: Option<CoordsRow> = sqlx::query_as(sql).bind(opp_id).fetch_optional(&self.pool).await?;
        Ok(row.map(|r| (r.longitude, r.latitude)))
    }

    pub async fn list_by_company(&self, company_id: Uuid, only_published: bool) -> Result<Vec<Opportunity>, StorageError> {
        let sql = if only_published {
            "SELECT * FROM opportunities WHERE company_id = $1 AND status = 'published' AND (expires_at IS NULL OR expires_at > NOW()) ORDER BY published_at DESC"
        } else {
            "SELECT * FROM opportunities WHERE company_id = $1 ORDER BY created_at DESC"
        };

        let rows = sqlx::query_as::<_, OpportunityDbRow>(sql).bind(company_id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|r| r.to_domain()).collect()
    }

    /// واکشی آگهی‌های منتشر شده شرکت به صورت غنی همراه با مشخصات شرکت و مختصات جغرافیایی شعبه در PostGIS
    pub async fn list_public_with_locations(
        &self,
        company_id: Uuid,
    ) -> Result<Vec<PublicOpportunityRow>, StorageError> {
        let sql = r#"
            SELECT 
                o.id,
                o.company_id,
                c.name AS company_name,
                c.slug AS company_slug,
                c.logo_storage_key AS company_logo,
                o.title,
                o.description,
                o.category_id,
                o.occupation_id,
                o.opportunity_type,
                o.workplace_type,
                o.remote_scope,
                o.experience_level,
                o.salary_min,
                o.salary_max,
                o.salary_currency,
                o.salary_period,
                o.status,
                o.published_at,
                o.expires_at,
                loc.id AS location_id,
                loc.address_summary AS location_summary,
                ST_X(loc.coordinates::geometry) AS longitude,
                ST_Y(loc.coordinates::geometry) AS latitude,
                o.created_at,
                o.updated_at
            FROM opportunities o
            INNER JOIN companies c ON c.id = o.company_id
            LEFT JOIN opportunity_locations ol ON ol.opportunity_id = o.id
            LEFT JOIN locations loc ON loc.id = ol.location_id
            WHERE o.company_id = $1 
              AND o.status = 'published' 
              AND (o.expires_at IS NULL OR o.expires_at > NOW())
            ORDER BY o.published_at DESC
        "#;

        let rows = sqlx::query_as::<_, PublicOpportunityRow>(sql)
            .bind(company_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows)
    }

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
                expires_at = COALESCE($4, expires_at),
                updated_at = NOW()
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