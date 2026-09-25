use crate::candidate_repo::CandidateMatchContext;
use crate::error::StorageError;
use biz_domain::discovery::{
    query::SearchCursor, CompanySummary, MapPinSummary, OpportunitySearchResult, SearchPageResult, SearchQuery, SortBy, SpatialContext,
};
use chrono::{DateTime, Utc};
use geo_types::{BoundingBox, GeoPoint, Radius};
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
    pub total_matching: Option<i64>,
    pub match_score: Option<i16>,
    pub matched_skills_count: Option<i64>,
    pub category_matches: Option<bool>,
    pub salary_matches: Option<bool>,
    pub workplace_matches: Option<bool>,
    pub location_matches: Option<bool>,
}

impl SearchDbRow {
    fn to_domain(self) -> OpportunitySearchResult {
        let coordinates = match (self.longitude, self.latitude) {
            (Some(lon), Some(lat)) => Some([lon, lat]),
            _ => None,
        };

        let mut match_reasons = Vec::new();
        if let Some(score) = self.match_score {
            if score >= 20 {
                if let Some(cnt) = self.matched_skills_count {
                    if cnt > 0 {
                        match_reasons.push(format!("تطابق {} مهارت تخصصی با رزومه شما", cnt));
                    }
                }
                if self.category_matches == Some(true) {
                    match_reasons.push("دسته‌بندی شغلی مورد علاقه شما".to_string());
                }
                if self.location_matches == Some(true) {
                    if self.workplace_type == "remote" {
                        match_reasons.push("امکان دورکاری از هر نقطه".to_string());
                    } else {
                        match_reasons.push("واقع در شهر سکونت یا شعاع تردد شما".to_string());
                    }
                }
                if self.workplace_matches == Some(true) {
                    match_reasons.push("شیوه کار منطبق با انتخاب شما".to_string());
                }
                if self.salary_matches == Some(true) {
                    match_reasons.push("حقوق پیشنهادی متناسب با انتظار مالی شما".to_string());
                }
            }
        }

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
            match_score: self.match_score.map(|s| s.clamp(0, 100) as u8),
            match_reasons,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MapPinDbRow {
    pub location_id: Uuid,
    pub longitude: f64,
    pub latitude: f64,
    pub address_summary: Option<String>,
    pub opportunity_count: i64,
    pub top_categories: Vec<String>,
    pub sample_companies: Vec<String>,
    pub min_salary: Option<Decimal>,
    pub max_salary: Option<Decimal>,
    pub salary_currency: String,
}

#[derive(Clone)]
pub struct DiscoveryRepository {
    pool: PgPool,
}

impl DiscoveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn search(
        &self,
        q: &SearchQuery,
        cand_ctx: Option<&CandidateMatchContext>,
        min_match_score: Option<u8>,
    ) -> Result<SearchPageResult, StorageError> {
        let limit = q.limit.clamp(1, 50);

        let (lon, lat) = q.point.map(|p| (Some(p.longitude()), Some(p.latitude()))).unwrap_or((None, None));
        let radius_m = q.radius.map(|r| Some(r.as_meters())).unwrap_or(None);
        let (west, south, east, north) = q.bbox
            .map(|b| (Some(b.west()), Some(b.south()), Some(b.east()), Some(b.north())))
            .unwrap_or((None, None, None, None));

        let (cursor_published_at, cursor_id) = q.cursor
            .map(|c| (Some(c.published_at), Some(c.id)))
            .unwrap_or((None, None));

        let is_radius_mode = radius_m.is_some() && lon.is_some() && lat.is_some();
        let is_bbox_mode = !is_radius_mode && west.is_some() && south.is_some() && east.is_some() && north.is_some();
        let is_city_mode = !is_radius_mode && !is_bbox_mode && q.city.is_some();

        let cand_skill_ids = cand_ctx.map(|c| c.skill_ids.as_slice()).unwrap_or(&[]);
        let cand_category_ids = cand_ctx.map(|c| c.preferred_category_ids.as_slice()).unwrap_or(&[]);
        let cand_city = cand_ctx.and_then(|c| c.preferred_city.as_deref());
        let cand_workplaces = cand_ctx.map(|c| c.preferred_workplace_types.as_slice()).unwrap_or(&[]);
        let cand_min_salary = cand_ctx.and_then(|c| c.expected_salary_min);
        let (cand_lon, cand_lat) = cand_ctx.and_then(|c| c.commute_coords).map(|(x, y)| (Some(x), Some(y))).unwrap_or((None, None));
        let cand_radius = cand_ctx.map(|c| c.commute_radius_meters as f64).unwrap_or(5000.0);
        let has_cand = cand_ctx.is_some();

        // گلوگاه اصلی ۱۰ ثانیه‌ای رفع شد: نمره‌دهی سنگین منحصراً وقتی اجرا می‌شود که کاربر نیاز به سورت سازگاری دارد
        let should_score = has_cand && (q.sort == SortBy::MatchScore || min_match_score.is_some());

        let order_clause = match q.sort {
            SortBy::MatchScore if has_cand => {
                "d.match_score DESC NULLS LAST, d.published_at DESC, d.id DESC"
            }
            SortBy::Distance if lon.is_some() && lat.is_some() => {
                "d.distance_meters ASC NULLS LAST, d.published_at DESC, d.id DESC"
            }
            SortBy::SalaryDesc => {
                "COALESCE(d.salary_max, d.salary_min, 0) DESC, d.published_at DESC, d.id DESC"
            }
            _ => "d.published_at DESC, d.id DESC",
        };

        let sql = format!(
            r#"
            WITH filtered_opportunities AS (
                SELECT DISTINCT ON (o.id)
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
                    END AS distance_meters,
                    -- فرمول تطبیق منحصراً در صورت نیاز اجرا می‌گردد
                    CASE 
                        WHEN $22::boolean = true THEN
                            (
                                SELECT 
                                    CASE 
                                        WHEN total_weight > 0.0 THEN
                                            LEAST(100, GREATEST(0, ROUND((earned_points / total_weight) * 100.0)))::int2
                                        ELSE 0::int2
                                    END
                                FROM (
                                    SELECT 
                                        (
                                            (CASE WHEN CARDINALITY($23::uuid[]) > 0 THEN 45.0 ELSE 0.0 END) +
                                            (CASE WHEN CARDINALITY($29::uuid[]) > 0 THEN 25.0 ELSE 0.0 END) +
                                            (CASE WHEN $26::text IS NOT NULL OR ($31::float8 IS NOT NULL AND $32::float8 IS NOT NULL) THEN 15.0 ELSE 0.0 END) +
                                            (CASE WHEN CARDINALITY($25::text[]) > 0 THEN 10.0 ELSE 0.0 END) +
                                            (CASE WHEN $24::numeric IS NOT NULL THEN 5.0 ELSE 0.0 END)
                                        )::float8 AS total_weight,
                                        (
                                            COALESCE(
                                                (SELECT (COUNT(DISTINCT os.skill_id)::float8 / NULLIF(COUNT(DISTINCT os2.skill_id), 0)) * 45.0
                                                 FROM opportunity_skills os2
                                                 LEFT JOIN opportunity_skills os ON os.opportunity_id = os2.opportunity_id AND os.skill_id = ANY($23)
                                                 WHERE os2.opportunity_id = o.id), 
                                                0.0
                                            ) +
                                            (CASE WHEN CARDINALITY($29::uuid[]) > 0 AND o.category_id = ANY($29) THEN 25.0 ELSE 0.0 END) +
                                            (CASE 
                                                WHEN o.workplace_type = 'remote' THEN 15.0
                                                WHEN $31::float8 IS NOT NULL AND $32::float8 IS NOT NULL AND loc.coordinates IS NOT NULL AND 
                                                     ST_Distance(loc.coordinates::geography, ST_SetSRID(ST_MakePoint($31, $32), 4326)::geography) <= ($30::float8 * 1.5) THEN 15.0
                                                WHEN $26::text IS NOT NULL AND loc.address_summary ILIKE '%' || $26 || '%' THEN 10.0
                                                ELSE 0.0 
                                             END) +
                                            (CASE WHEN CARDINALITY($25::text[]) > 0 AND o.workplace_type = ANY($25) THEN 10.0 ELSE 0.0 END) +
                                            (CASE WHEN $24::numeric IS NOT NULL AND COALESCE(o.salary_max, o.salary_min) >= $24 THEN 5.0 ELSE 0.0 END)
                                        )::float8 AS earned_points
                                ) scoring_calc
                            )
                        ELSE NULL 
                    END AS match_score,
                    CASE 
                        WHEN $22::boolean = true THEN
                            (SELECT COUNT(DISTINCT os.skill_id) FROM opportunity_skills os WHERE os.opportunity_id = o.id AND os.skill_id = ANY($23))
                        ELSE NULL 
                    END AS matched_skills_count,
                    (CASE WHEN CARDINALITY($29::uuid[]) > 0 AND o.category_id = ANY($29) THEN true ELSE false END) AS category_matches,
                    (CASE WHEN $24::numeric IS NOT NULL AND COALESCE(o.salary_max, o.salary_min) >= $24 THEN true ELSE false END) AS salary_matches,
                    (CASE WHEN CARDINALITY($25::text[]) > 0 AND o.workplace_type = ANY($25) THEN true ELSE false END) AS workplace_matches,
                    (CASE 
                        WHEN o.workplace_type = 'remote' THEN true
                        WHEN $26::text IS NOT NULL AND loc.address_summary ILIKE '%' || $26 || '%' THEN true
                        WHEN $31::float8 IS NOT NULL AND loc.coordinates IS NOT NULL AND ST_Distance(loc.coordinates::geography, ST_SetSRID(ST_MakePoint($31, $32), 4326)::geography) <= ($30::float8 * 1.5) THEN true
                        ELSE false 
                     END) AS location_matches
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
                  AND ($8::numeric IS NULL OR COALESCE(o.salary_max, o.salary_min) >= $8)
                  AND ($28::numeric IS NULL OR COALESCE(o.salary_min, o.salary_max) <= $28)
                  AND (
                      $18::uuid[] IS NULL OR CARDINALITY($18) = 0 OR
                      EXISTS (
                          SELECT 1 FROM opportunity_skills os
                          WHERE os.opportunity_id = o.id AND os.skill_id = ANY($18)
                      )
                  )
                  AND ($19::boolean = true OR o.workplace_type != 'remote')
                  AND (
                      $15::float8 IS NULL OR
                      (loc.coordinates IS NOT NULL AND ST_DWithin(loc.coordinates::geography, ST_SetSRID(ST_MakePoint($13, $14), 4326)::geography, $15))
                  )
                  AND (
                      $9::float8 IS NULL OR
                      (loc.coordinates IS NOT NULL AND (loc.coordinates && ST_MakeEnvelope($9, $10, $11, $12, 4326)))
                  )
                  AND (
                      $21::text IS NULL OR
                      (loc.address_summary ILIKE '%' || $21 || '%')
                  )
                ORDER BY o.id, distance_meters ASC NULLS LAST
            ),
            counted_total AS (
                SELECT COUNT(*)::int8 AS total_matching 
                FROM filtered_opportunities d
                WHERE ($27::int2 IS NULL OR d.match_score >= $27)
            )
            SELECT 
                d.*,
                c.total_matching
            FROM filtered_opportunities d
            CROSS JOIN counted_total c
            WHERE ($16::timestamptz IS NULL OR (d.published_at, d.id) < ($16, $17))
              AND ($27::int2 IS NULL OR d.match_score >= $27)
            ORDER BY {order_clause}
            LIMIT $20
            "#
        );

        let effective_west = if is_bbox_mode { west } else { None };
        let effective_south = if is_bbox_mode { south } else { None };
        let effective_east = if is_bbox_mode { east } else { None };
        let effective_north = if is_bbox_mode { north } else { None };
        let effective_city = if is_city_mode { q.city.as_deref() } else { None };
        let effective_radius = if is_radius_mode { radius_m } else { None };
        let min_score_i16 = min_match_score.map(|s| s as i16);

        let rows = sqlx::query_as::<_, SearchDbRow>(&sql)
            .bind(&q.text)               // $1
            .bind(q.category_id)         // $2
            .bind(q.occupation_id)       // $3
            .bind(q.company_id)          // $4
            .bind(&q.opportunity_type)   // $5
            .bind(&q.workplace_type)     // $6
            .bind(&q.experience_level)   // $7
            .bind(q.salary_min)          // $8
            .bind(effective_west)        // $9
            .bind(effective_south)       // $10
            .bind(effective_east)        // $11
            .bind(effective_north)       // $12
            .bind(lon)                   // $13
            .bind(lat)                   // $14
            .bind(effective_radius)      // $15
            .bind(cursor_published_at)   // $16
            .bind(cursor_id)             // $17
            .bind(&q.skill_ids)          // $18
            .bind(q.include_remote)      // $19
            .bind((limit + 1) as i64)    // $20
            .bind(effective_city)        // $21
            .bind(should_score)          // $22 (بهینه‌سازی شده)
            .bind(cand_skill_ids)        // $23
            .bind(cand_min_salary)       // $24
            .bind(cand_workplaces)       // $25
            .bind(cand_city)             // $26
            .bind(min_score_i16)         // $27
            .bind(q.salary_max)          // $28
            .bind(cand_category_ids)     // $29
            .bind(cand_radius)           // $30
            .bind(cand_lon)              // $31
            .bind(cand_lat)              // $32
            .fetch_all(&self.pool)
            .await?;

        let total_count = rows.first().and_then(|r| r.total_matching).unwrap_or(0);
        let has_more = rows.len() > limit;
        let items: Vec<OpportunitySearchResult> = rows.into_iter().take(limit).map(SearchDbRow::to_domain).collect();

        let next_cursor = if has_more {
            items.last().and_then(|it| {
                it.published_at.map(|pub_at| SearchCursor { published_at: pub_at, id: it.id }.encode())
            })
        } else {
            None
        };

        let national_total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM opportunities WHERE status = 'published' AND (expires_at IS NULL OR expires_at > NOW())"
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        let (scope_str, detected_city_name) = if is_radius_mode {
            ("radius", None)
        } else if is_city_mode {
            ("city", q.city.clone())
        } else if is_bbox_mode {
            let city_from_db: Option<String> = sqlx::query_scalar(
                r#"
                SELECT TRIM(split_part(loc.address_summary, '،', 1)) AS city_name
                FROM locations loc
                WHERE loc.coordinates && ST_MakeEnvelope($1, $2, $3, $4, 4326)
                  AND loc.address_summary IS NOT NULL
                GROUP BY city_name
                ORDER BY COUNT(*) DESC
                LIMIT 1
                "#
            )
            .bind(west.unwrap())
            .bind(south.unwrap())
            .bind(east.unwrap())
            .bind(north.unwrap())
            .fetch_optional(&self.pool)
            .await
            .unwrap_or(None);

            ("viewport", city_from_db)
        } else {
            ("national", None)
        };

        let city_total_jobs = if let Some(ref c_name) = detected_city_name {
            let count: i64 = sqlx::query_scalar(
                r#"
                SELECT COUNT(DISTINCT o.id)
                FROM opportunities o
                INNER JOIN opportunity_locations ol ON ol.opportunity_id = o.id
                INNER JOIN locations loc ON loc.id = ol.location_id
                WHERE o.status = 'published'
                  AND (o.expires_at IS NULL OR o.expires_at > NOW())
                  AND (loc.address_summary ILIKE '%' || $1 || '%')
                "#
            )
            .bind(c_name)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
            Some(count)
        } else {
            None
        };

        let spatial_context = SpatialContext {
            scope: scope_str.to_string(),
            detected_city: detected_city_name,
            city_total_jobs,
            national_total_jobs: national_total,
        };

        Ok(SearchPageResult {
            items,
            total_count,
            next_cursor,
            has_more,
            spatial_context,
        })
    }

    pub async fn list_map_pins(
        &self,
        q_text: Option<&str>,
        bbox: Option<&BoundingBox>,
        point: Option<&GeoPoint>,
        radius: Option<&Radius>,
        city: Option<&str>,
        category_id: Option<Uuid>,
        skill_ids: &[Uuid],
        opportunity_type: Option<&str>,
        workplace_type: Option<&str>,
        experience_level: Option<&str>,
        salary_min: Option<Decimal>,
        salary_max: Option<Decimal>,
        limit: usize,
    ) -> Result<Vec<MapPinSummary>, StorageError> {
        let (lon, lat) = point
            .map(|p| (Some(p.longitude()), Some(p.latitude())))
            .unwrap_or((None, None));

        let radius_m = radius.map(|r| Some(r.as_meters())).unwrap_or(None);
        let is_radius_mode = radius_m.is_some() && lon.is_some() && lat.is_some();

        let (west, south, east, north) = if !is_radius_mode {
            bbox.map(|b| (Some(b.west()), Some(b.south()), Some(b.east()), Some(b.north())))
                .unwrap_or((None, None, None, None))
        } else {
            (None, None, None, None)
        };

        let is_bbox_mode = !is_radius_mode && west.is_some();
        let effective_city = if !is_radius_mode && !is_bbox_mode { city } else { None };

        let sql = r#"
            SELECT 
                loc.id AS location_id,
                ST_X(loc.coordinates::geometry) AS longitude,
                ST_Y(loc.coordinates::geometry) AS latitude,
                loc.address_summary,
                COUNT(DISTINCT o.id)::int8 AS opportunity_count,
                COALESCE((ARRAY_AGG(DISTINCT cat.name) FILTER (WHERE cat.name IS NOT NULL))[1:3], ARRAY[]::text[]) AS top_categories,
                COALESCE((ARRAY_AGG(DISTINCT c.name) FILTER (WHERE c.name IS NOT NULL))[1:3], ARRAY[]::text[]) AS sample_companies,
                MIN(o.salary_min) AS min_salary,
                MAX(o.salary_max) AS max_salary,
                COALESCE(MAX(o.salary_currency), 'IRR') AS salary_currency
            FROM locations loc
            INNER JOIN opportunity_locations ol ON ol.location_id = loc.id
            INNER JOIN opportunities o ON o.id = ol.opportunity_id
            INNER JOIN companies c ON c.id = o.company_id
            LEFT JOIN categories cat ON cat.id = o.category_id
            WHERE o.status = 'published'
              AND (o.expires_at IS NULL OR o.expires_at > NOW())
              AND ($1::text IS NULL OR o.search_vector @@ plainto_tsquery('simple', $1))
              AND ($2::float8 IS NULL OR (loc.coordinates && ST_MakeEnvelope($2, $3, $4, $5, 4326)))
              AND ($6::float8 IS NULL OR ST_DWithin(loc.coordinates::geography, ST_SetSRID(ST_MakePoint($7, $8), 4326)::geography, $6))
              AND ($9::text IS NULL OR loc.address_summary ILIKE '%' || $9 || '%')
              AND ($10::uuid IS NULL OR o.category_id = $10)
              AND ($11::text IS NULL OR o.opportunity_type = $11)
              AND ($12::text IS NULL OR o.workplace_type = $12)
              AND ($13::text IS NULL OR o.experience_level = $13)
              AND ($14::numeric IS NULL OR COALESCE(o.salary_max, o.salary_min) >= $14)
              AND ($15::numeric IS NULL OR COALESCE(o.salary_min, o.salary_max) <= $15)
              AND (
                  $16::uuid[] IS NULL OR CARDINALITY($16) = 0 OR
                  EXISTS (
                      SELECT 1 FROM opportunity_skills os
                      WHERE os.opportunity_id = o.id AND os.skill_id = ANY($16)
                  )
              )
            GROUP BY loc.id, loc.coordinates, loc.address_summary
            ORDER BY opportunity_count DESC
            LIMIT $17
        "#;

        let rows = sqlx::query_as::<_, MapPinDbRow>(sql)
            .bind(q_text)
            .bind(west)
            .bind(south)
            .bind(east)
            .bind(north)
            .bind(if is_radius_mode { radius_m } else { None })
            .bind(lon)
            .bind(lat)
            .bind(effective_city)
            .bind(category_id)
            .bind(opportunity_type)
            .bind(workplace_type)
            .bind(experience_level)
            .bind(salary_min)
            .bind(salary_max)
            .bind(skill_ids)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| MapPinSummary {
                location_id: r.location_id,
                coordinates: [r.longitude, r.latitude],
                address_summary: r.address_summary,
                opportunity_count: r.opportunity_count,
                top_categories: r.top_categories,
                sample_companies: r.sample_companies,
                min_salary: r.min_salary,
                max_salary: r.max_salary,
                salary_currency: r.salary_currency,
            })
            .collect())
    }

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
                NULL::float8 AS distance_meters,
                NULL::int8 AS total_matching,
                NULL::int2 AS match_score,
                NULL::int8 AS matched_skills_count,
                NULL::boolean AS category_matches,
                NULL::boolean AS salary_matches,
                NULL::boolean AS workplace_matches,
                NULL::boolean AS location_matches
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