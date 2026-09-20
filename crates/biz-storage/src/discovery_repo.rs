use crate::error::StorageError;
use biz_domain::discovery::{CompanySummary, OpportunitySearchResult, SearchPageResult, SearchQuery};
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
    pub location_id: Option<Uuid>,
    pub location_summary: Option<String>,
    pub longitude: Option<f64>,
    pub latitude: Option<f64>,
    pub distance_meters: Option<f64>,
}

impl SearchDbRow {
    fn to_domain(self) -> OpportunitySearchResult {
        let coordinates = match (self.longitude, self.latitude) {
            (Some(lon), Some(lat)) => Some([lon, lat]),
            _ => None,
        };

        OpportunitySearchResult {
            id: self.id,
            title: self.title,
            description_summary: self.description,
            opportunity_type: self.opportunity_type,
            workplace_type: self.workplace_type,
            remote_scope: self.remote_scope,
            experience_level: self.experience_level,
            salary_min: self.salary_min,
            salary_max: self.salary_max,
            salary_currency: self.salary_currency,
            salary_period: self.salary_period,
            published_at: self.published_at,
            company: CompanySummary {
                id: self.company_id,
                name: self.company_name,
                slug: self.company_slug,
                logo_storage_key: self.company_logo,
            },
            location_id: self.location_id,
            location_summary: self.location_summary,
            coordinates,
            distance_meters: self.distance_meters,
        }
    }
}

#[derive(Clone)]
pub struct DiscoveryRepository {
    pool: PgPool,
}

impl DiscoveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search(&self, q: &SearchQuery) -> Result<SearchPageResult, StorageError> {
        let limit = q.limit.clamp(1, 50);

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
                ST_Y(loc.coordinates::geometry) AS latitude,
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
              AND ($1::text IS NULL OR o.search_vector @@ plainto_tsquery('simple', $1))
              AND ($2::uuid IS NULL OR o.category_id = $2)
              AND ($3::uuid IS NULL OR o.occupation_id = $3)
              AND ($4::uuid IS NULL OR o.company_id = $4)
              AND ($5::text IS NULL OR o.opportunity_type = $5)
              AND ($6::text IS NULL OR o.workplace_type = $6)
              AND ($7::text IS NULL OR o.experience_level = $7)
              AND ($8::numeric IS NULL OR o.salary_max >= $8)
              AND ($9::float8 IS NULL OR (loc.coordinates && ST_MakeEnvelope($9, $10, $11, $12, 4326)))
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
            .bind((limit + 1) as i64)
            .fetch_all(&self.pool)
            .await?;

        let has_more = rows.len() > limit;
        let items: Vec<OpportunitySearchResult> = rows.into_iter().take(limit).map(SearchDbRow::to_domain).collect();

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

    /// واکشی سریع تمام آگهی‌های فعال متعلق به یک پین یا مارکر خاص روی نقشه
    pub async fn list_by_location(&self, location_id: Uuid) -> Result<Vec<OpportunitySearchResult>, StorageError> {
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
                ST_Y(loc.coordinates::geometry) AS latitude,
                NULL::float8 AS distance_meters
            FROM opportunities o
            INNER JOIN companies c ON c.id = o.company_id
            INNER JOIN opportunity_locations ol ON ol.opportunity_id = o.id
            INNER JOIN locations loc ON loc.id = ol.location_id
            WHERE ol.location_id = $1
              AND o.status = 'published'
              AND (o.expires_at IS NULL OR o.expires_at > NOW())
            ORDER BY o.published_at DESC
        "#;

        let rows = sqlx::query_as::<_, SearchDbRow>(sql)
            .bind(location_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(SearchDbRow::to_domain).collect())
    }
}