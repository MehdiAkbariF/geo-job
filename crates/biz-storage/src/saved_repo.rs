use crate::error::StorageError;
use biz_domain::discovery::{CompanySummary, OpportunitySearchResult};
use biz_domain::saved::{CandidatePreferences, JobRadar, Notification, SavedCompany, SavedOpportunity, SavedSearch};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct RadarDbRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    longitude: Option<f64>,
    latitude: Option<f64>,
    radius_meters: i32,
    keywords: Option<String>,
    min_salary: Option<Decimal>,
    workplace_type: Option<String>,
    category_id: Option<Uuid>,
    is_active: bool,
    notify_in_app: bool,
    notify_sms: bool,
    last_triggered_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct NotificationDbRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    message: String,
    action_url: Option<String>,
    notification_type: String,
    is_read: bool,
    metadata: serde_json::Value,
    created_at: DateTime<Utc>,
}

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

    // ==========================================
    // ۱. متدهای بوک‌مارک آگهی و شرکت (ذخیره‌شده‌ها)
    // ==========================================

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

    // ==========================================
    // ۲. سیستم رادارهای هوشمند و اعلان‌ها
    // ==========================================
/// ایجاد رادار شغلی ژئوفنس روی نقشه
    pub async fn create_radar(
        &self,
        user_id: Uuid,
        title: &str,
        coords: Option<[f64; 2]>,
        radius_meters: i32,
        keywords: Option<&str>,
        min_salary: Option<Decimal>,
        workplace_type: Option<&str>,
        category_id: Option<Uuid>,
    ) -> Result<Uuid, StorageError> {
        let (lon, lat) = coords.map(|c| (Some(c[0]), Some(c[1]))).unwrap_or((None, None));

        let sql = r#"
            INSERT INTO saved_searches (
                user_id, candidate_id, title, center_point, radius_meters, keywords,
                min_salary, workplace_type, category_id, criteria, is_active,
                notify_in_app, notify_sms
            )
            VALUES (
                $1,
                (SELECT id FROM candidates WHERE user_id = $1 LIMIT 1),
                $2,
                CASE WHEN $3::float8 IS NOT NULL AND $4::float8 IS NOT NULL THEN
                    ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography
                ELSE NULL END,
                $5, $6, $7, $8, $9, '{}'::jsonb, true, true, false
            )
            RETURNING id
        "#;

        let id: Uuid = sqlx::query_scalar(sql)
            .bind(user_id)
            .bind(title)
            .bind(lon)
            .bind(lat)
            .bind(radius_meters)
            .bind(keywords)
            .bind(min_salary)
            .bind(workplace_type)
            .bind(category_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(id)
    }

    /// لیست رادارهای کاربر
    pub async fn list_user_radars(&self, user_id: Uuid) -> Result<Vec<JobRadar>, StorageError> {
        let sql = r#"
            SELECT 
                s.id,
                COALESCE(s.user_id, c.user_id) AS user_id,
                s.title,
                ST_X(s.center_point::geometry) AS longitude,
                ST_Y(s.center_point::geometry) AS latitude,
                s.radius_meters,
                s.keywords,
                s.min_salary,
                s.workplace_type,
                s.category_id,
                s.is_active,
                s.notify_in_app,
                s.notify_sms,
                s.last_triggered_at,
                s.created_at
            FROM saved_searches s
            LEFT JOIN candidates c ON c.id = s.candidate_id
            WHERE s.user_id = $1 
               OR s.candidate_id IN (SELECT id FROM candidates WHERE user_id = $1)
            ORDER BY s.created_at DESC
        "#;

        let rows = sqlx::query_as::<_, RadarDbRow>(sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| {
            let center_coordinates = match (r.longitude, r.latitude) {
                (Some(lon), Some(lat)) => Some([lon, lat]),
                _ => None,
            };

            JobRadar {
                id: r.id,
                user_id: r.user_id,
                title: r.title,
                center_coordinates,
                radius_meters: r.radius_meters,
                keywords: r.keywords,
                min_salary: r.min_salary,
                workplace_type: r.workplace_type,
                category_id: r.category_id,
                is_active: r.is_active,
                notify_in_app: r.notify_in_app,
                notify_sms: r.notify_sms,
                last_triggered_at: r.last_triggered_at,
                created_at: r.created_at,
            }
        }).collect())
    }

    /// حذف رادار
    pub async fn delete_radar(&self, user_id: Uuid, radar_id: Uuid) -> Result<(), StorageError> {
        let sql = r#"
            DELETE FROM saved_searches 
            WHERE id = $1 
              AND (user_id = $2 OR candidate_id IN (SELECT id FROM candidates WHERE user_id = $2))
        "#;

        sqlx::query(sql)
            .bind(radar_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// مانیتورینگ فضایی در زمان انتشار آگهی و ثبت اعلان برای رادارها
    pub async fn match_radars_and_notify_on_job_publish(
        &self,
        opportunity_id: Uuid,
    ) -> Result<usize, StorageError> {
        let sql = r#"
            WITH new_job AS (
                SELECT 
                    o.id,
                    o.title,
                    o.category_id,
                    o.workplace_type,
                    COALESCE(o.salary_max, o.salary_min, 0) AS job_salary,
                    c.name AS company_name,
                    loc.coordinates AS job_geom,
                    loc.address_summary
                FROM opportunities o
                INNER JOIN companies c ON c.id = o.company_id
                LEFT JOIN opportunity_locations ol ON ol.opportunity_id = o.id
                LEFT JOIN locations loc ON loc.id = ol.location_id
                WHERE o.id = $1
                LIMIT 1
            ),
            matching_radars AS (
                SELECT 
                    r.id AS radar_id,
                    COALESCE(r.user_id, cand.user_id) AS user_id,
                    r.title AS radar_title,
                    j.id AS opportunity_id,
                    j.title AS job_title,
                    j.company_name,
                    j.address_summary,
                    ROUND(ST_Distance(r.center_point, j.job_geom)::numeric / 1000.0, 1) AS dist_km
                FROM saved_searches r
                LEFT JOIN candidates cand ON cand.id = r.candidate_id
                CROSS JOIN new_job j
                WHERE r.is_active = true
                  AND (r.category_id IS NULL OR r.category_id = j.category_id)
                  AND (r.workplace_type IS NULL OR r.workplace_type = j.workplace_type)
                  AND (r.min_salary IS NULL OR j.job_salary >= r.min_salary)
                  AND (r.keywords IS NULL OR j.title ILIKE '%' || r.keywords || '%')
                  AND (
                      j.workplace_type = 'remote'
                      OR (
                          r.center_point IS NOT NULL 
                          AND j.job_geom IS NOT NULL 
                          AND ST_DWithin(r.center_point, j.job_geom, r.radius_meters)
                      )
                  )
                  AND (r.last_triggered_at IS NULL OR r.last_triggered_at < NOW() - INTERVAL '1 hour')
            ),
            inserted_notifications AS (
                INSERT INTO notifications (user_id, title, message, action_url, notification_type, metadata)
                SELECT 
                    m.user_id,
                    '🎯 رادار شغلی فعال شد: ' || m.job_title,
                    'فرصت شغلی جدید در ' || COALESCE(m.company_name, 'یک شرکت') || 
                    CASE WHEN m.dist_km IS NOT NULL THEN ' (در فاصله ' || m.dist_km || ' کیلومتری شما)' ELSE '' END || 
                    ' با معیارهای رادار «' || m.radar_title || '» شما منطبق است.',
                    '/opportunities/' || m.opportunity_id,
                    'radar_match',
                    jsonb_build_object('opportunity_id', m.opportunity_id, 'radar_id', m.radar_id)
                FROM matching_radars m
                WHERE m.user_id IS NOT NULL
                RETURNING id
            ),
            updated_radars AS (
                UPDATE saved_searches
                SET last_triggered_at = NOW()
                WHERE id IN (SELECT radar_id FROM matching_radars)
                RETURNING id
            )
            SELECT COUNT(*)::int8 FROM inserted_notifications;
        "#;

        let notified_count: i64 = sqlx::query_scalar(sql)
            .bind(opportunity_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(notified_count as usize)
    }
    pub async fn list_user_notifications(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Notification>, StorageError> {
        let sql = r#"
            SELECT id, user_id, title, message, action_url, notification_type, is_read, metadata, created_at
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
        "#;

        let rows = sqlx::query_as::<_, NotificationDbRow>(sql)
            .bind(user_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| Notification {
            id: r.id,
            user_id: r.user_id,
            title: r.title,
            message: r.message,
            action_url: r.action_url,
            notification_type: r.notification_type,
            is_read: r.is_read,
            metadata: r.metadata,
            created_at: r.created_at,
        }).collect())
    }

    pub async fn mark_notification_read(&self, user_id: Uuid, notification_id: Uuid) -> Result<(), StorageError> {
        sqlx::query("UPDATE notifications SET is_read = true WHERE id = $1 AND user_id = $2")
            .bind(notification_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_unread_notifications_count(&self, user_id: Uuid) -> Result<i64, StorageError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}