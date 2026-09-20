use crate::error::StorageError;
use biz_domain::discovery::{CompanySummary, OpportunitySearchResult, SearchPageResult, SearchQuery, SortBy};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct SearchDbRow {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub opportunity_type: String,
    pub workplace_type: String,
    pub remote_scope: Option<String>,
    pub experience_level: String,
    pub salary_min: Option<Decimal>,
    pub salary_max: Option<Decimal>,
    pub salary_currency: String,
    pub salary_period: String,
    pub published_at: Option<DateTime<Utc>>,
    pub company_id: Uuid,
    pub company_name: String,
    pub company_slug: String,
    pub company_logo: Option<String>,
    pub location_summary: Option<String>,
    pub distance_meters: Option<f64>,
}

#[derive(Clone)]
pub struct DiscoveryRepository {
    pool: PgPool,
}

impl DiscoveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Orchestrates PostgreSQL Full-Text Search (GIN) + PostGIS (GiST) + Server-side Visibility
    pub async fn search(&self, q: &SearchQuery) -> Result<SearchPageResult, StorageError> {
        let limit = q.limit.clamp(1, 50);

        // SQL Query combining relational filters, FTS GIN, and PostGIS GiST spatial joins
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
                loc.address_summary AS location_summary,
                CASE 
                    WHEN $13::float8 IS NOT NULL AND $14::float8 IS NOT NULL AND loc.coordinates IS NOT NULL THEN
                        ST_Distance(loc.coordinates::geography, ST_SetSRID(ST_MakePoint($13, $14), 4326)::geography)
                    ELSE NULL
                END AS distance_meters
            FROM opportunities o
            INNER JOIN companies c ON c.id = o.company_id
            LEFT JOIN opportunity_locations ol ON ol.opportunity_id = o.id
            LEFT JOIN locations loc ON loc.id = ol.location_id
            WHERE o.status = 'published'
              AND (o.expires_at IS NULL OR o.expires_at > NOW())
              -- Full-Text Search (FTS) using GIN index
              AND ($1::text IS NULL OR o.search_vector @@ plainto_tsquery('simple', $1))
              -- Taxonomy Filters
              AND ($2::uuid IS NULL OR o.category_id = $2)
              AND ($3::uuid IS NULL OR o.occupation_id = $3)
              AND ($4::uuid IS NULL OR o.company_id = $4)
              AND ($5::text IS NULL OR o.opportunity_type = $5)
              AND ($6::text IS NULL OR o.workplace_type = $6)
              AND ($7::text IS NULL OR o.experience_level = $7)
              AND ($8::numeric IS NULL OR o.salary_max >= $8)
              -- Spatial Bounding Box Filter (PostGIS GiST)
              AND ($9::float8 IS NULL OR (loc.coordinates && ST_MakeEnvelope($9, $10, $11, $12, 4326)))
              -- Spatial Radius Filter (PostGIS GiST on geography in meters)
              AND ($15::float8 IS NULL OR ST_DWithin(loc.coordinates::geography, ST_SetSRID(ST_MakePoint($13, $14), 4326)::geography, $15))
            ORDER BY o.published_at DESC, o.id DESC
            LIMIT $16
        "#;

        let (lon, lat) = q.point.map(|p| (Some(p.longitude()), Some(p.latitude()))).unwrap_or((None, None));
        let radius_m = q.radius.map(|r| Some(r.as_meters())).unwrap_or(None);
        let (west, south, east, north) = q.bbox.map(|b| (Some(b.west()), Some(b.south()), Some(b.east()), Some(b.north())))
            .unwrap_or((None, None, None, None));

        let rows = sqlx::query_as::<_, SearchDbRow>(sql)
            .bind(&q.text)
            .bind(q.category_id)
            .bind(q.occupation_id)
            .bind(q.company_id)
            .bind(&q.opportunity_type)
            .bind(&q.workplace_type)
            .bind(&q.experience_level)
            .bind(q.salary_min)
            .bind(west)
            .bind(south)
            .bind(east)
            .bind(north)
            .bind(lon)
            .bind(lat)
            .bind(radius_m)
            .bind((limit + 1) as i64) // Fetch 1 extra to determine if has_more exists
            .fetch_all(&self.pool)
            .await?;

        let has_more = rows.len() > limit;
        let items: Vec<OpportunitySearchResult> = rows.into_iter().take(limit).map(|r| {
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
                company: CompanySummary {
                    id: r.company_id,
                    name: r.company_name,
                    slug: r.company_slug,
                    logo_storage_key: r.company_logo,
                },
                location_summary: r.location_summary,
                distance_meters: r.distance_meters,
            }
        }).collect();

        let next_cursor = if has_more {
            items.last().map(|it| it.id.to_string())
        } else {
            None
        };

        Ok(SearchPageResult {
            items,
            next_cursor,
            has_more,
        })
    }
}