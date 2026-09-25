use crate::error::StorageError;
use biz_domain::discovery::{CompanySummary, OpportunitySearchResult};
use biz_domain::saved::{CandidatePreferences, SavedCompany, SavedOpportunity, SavedSearch};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct SavedSearchDbRow {
    id: Uuid,
    candidate_id: Uuid,
    title: String,
    criteria: serde_json::Value,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct SavedOppDbRow {
    id: Uuid,
    title: String,
    description: String,
    opportunity_type: String,
    workplace_type: String,
    remote_scope: Option<String>,
    experience_level: String,
    salary_min: Option<Decimal>,
    salary_max: Option<Decimal>,
    salary_currency: String,
    salary_period: String,
    published_at: Option<DateTime<Utc>>,
    company_id: Uuid,
    company_name: String,
    company_slug: String,
    company_logo: Option<String>,
    location_id: Option<Uuid>,
    location_summary: Option<String>,
    longitude: Option<f64>,
    latitude: Option<f64>,
}

#[derive(Clone)]
pub struct SavedRepository {
    pool: PgPool,
}

impl SavedRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_opportunity(&self, candidate_id: Uuid, opportunity_id: Uuid) -> Result<(), StorageError> {
        let sql = "INSERT INTO saved_opportunities (candidate_id, opportunity_id) VALUES ($1, $2) ON CONFLICT DO NOTHING";
        sqlx::query(sql)
            .bind(candidate_id)
            .bind(opportunity_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn remove_saved_opportunity(&self, candidate_id: Uuid, opportunity_id: Uuid) -> Result<(), StorageError> {
        let sql = "DELETE FROM saved_opportunities WHERE candidate_id = $1 AND opportunity_id = $2";
        sqlx::query(sql)
            .bind(candidate_id)
            .bind(opportunity_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_saved_opportunities(&self, candidate_id: Uuid) -> Result<Vec<OpportunitySearchResult>, StorageError> {
        let sql = r#"
            SELECT 
                o.id,
                o.title,
                SUBSTRING(o.description FROM 1 FOR 180) AS description,
                o.opportunity_type,
                o.workplace_type,
                o.remote_scope,
                o.experience_level,
                o.salary_min,
                o.salary_max,
                o.salary_currency,
                o.salary_period,
                o.published_at,
                c.id AS company_id,
                c.name AS company_name,
                c.slug AS company_slug,
                c.logo_storage_key AS company_logo,
                loc.id AS location_id,
                loc.address_summary AS location_summary,
                ST_X(loc.coordinates::geometry) AS longitude,
                ST_Y(loc.coordinates::geometry) AS latitude
            FROM saved_opportunities so
            INNER JOIN opportunities o ON o.id = so.opportunity_id
            INNER JOIN companies c ON c.id = o.company_id
            LEFT JOIN opportunity_locations ol ON ol.opportunity_id = o.id
            LEFT JOIN locations loc ON loc.id = ol.location_id
            WHERE so.candidate_id = $1
            ORDER BY so.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, SavedOppDbRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| {
            let coords = match (r.longitude, r.latitude) {
                (Some(lon), Some(lat)) => Some([lon, lat]),
                _ => None,
            };

          OpportunitySearchResult {
                id: r.id,
                title: r.title,
                description_summary: r.description,
                opportunity_type: r.opportunity_type,
                workplace_type: r.workplace_type,
                remote_scope: r.remote_scope,
                experience_level: r.experience_level,
                salary_min: r.salary_min,
                salary_max: r.salary_max,
                salary_currency: r.salary_currency,
                salary_period: r.salary_period,
                published_at: r.published_at,
                is_urgent: false,     // ✅
                is_featured: false,   // ✅
                company: CompanySummary {
                    id: r.company_id,
                    name: r.company_name,
                    slug: r.company_slug,
                    logo_storage_key: r.company_logo,
                },
                location_id: r.location_id,
                location_summary: r.location_summary,
                coordinates: coords,
                distance_meters: None,
                match_score: None,
                match_reasons: Vec::new(),
            }
        }).collect())
    }

    pub async fn save_company(&self, candidate_id: Uuid, company_id: Uuid) -> Result<(), StorageError> {
        let sql = "INSERT INTO saved_companies (candidate_id, company_id) VALUES ($1, $2) ON CONFLICT DO NOTHING";
        sqlx::query(sql)
            .bind(candidate_id)
            .bind(company_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn remove_saved_company(&self, candidate_id: Uuid, company_id: Uuid) -> Result<(), StorageError> {
        let sql = "DELETE FROM saved_companies WHERE candidate_id = $1 AND company_id = $2";
        sqlx::query(sql)
            .bind(candidate_id)
            .bind(company_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn list_saved_companies(&self, candidate_id: Uuid) -> Result<Vec<CompanySummary>, StorageError> {
        #[derive(sqlx::FromRow)]
        struct CompRow {
            id: Uuid,
            name: String,
            slug: String,
            logo_storage_key: Option<String>,
        }

        let sql = r#"
            SELECT c.id, c.name, c.slug, c.logo_storage_key
            FROM saved_companies sc
            INNER JOIN companies c ON c.id = sc.company_id
            WHERE sc.candidate_id = $1
            ORDER BY sc.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, CompRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| CompanySummary {
            id: r.id,
            name: r.name,
            slug: r.slug,
            logo_storage_key: r.logo_storage_key,
        }).collect())
    }

    pub async fn create_saved_search(
        &self,
        candidate_id: Uuid,
        title: &str,
        criteria: &serde_json::Value,
    ) -> Result<Uuid, StorageError> {
        let sql = r#"
            INSERT INTO saved_searches (candidate_id, title, criteria)
            VALUES ($1, $2, $3)
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(candidate_id)
            .bind(title)
            .bind(criteria)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    pub async fn list_saved_searches(&self, candidate_id: Uuid) -> Result<Vec<SavedSearch>, StorageError> {
        let sql = "SELECT id, candidate_id, title, criteria, created_at FROM saved_searches WHERE candidate_id = $1 ORDER BY created_at DESC";
        let rows = sqlx::query_as::<_, SavedSearchDbRow>(sql)
            .bind(candidate_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| SavedSearch {
            id: r.id,
            candidate_id: r.candidate_id,
            title: r.title,
            criteria: r.criteria,
            created_at: r.created_at,
        }).collect())
    }
}