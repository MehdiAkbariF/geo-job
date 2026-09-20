use crate::admin_repo::AdminAreaDbRow;
use crate::error::GeocodingError;
use crate::models::{AdministrativeArea, AreaBreadcrumb, ReverseGeocodeResult};
use geo_types::GeoPoint;
use sqlx::PgPool;

pub async fn reverse_geocode_point(
    pool: &PgPool,
    point: &GeoPoint,
) -> Result<Option<ReverseGeocodeResult>, GeocodingError> {
    // ۱. اولویت اول: بررسی چندضلعی‌های اداری (در صورت وجود در جدول administrative_areas)
    let sql_poly = r#"
        SELECT 
            id,
            parent_id,
            country_code,
            admin_level,
            area_type,
            name,
            name_en,
            ST_X(center::geometry) AS center_lon,
            ST_Y(center::geometry) AS center_lat,
            metadata
        FROM administrative_areas
        WHERE ST_Contains(boundary, ST_SetSRID(ST_MakePoint($1, $2), 4326))
        ORDER BY admin_level DESC
    "#;

    let rows = sqlx::query_as::<_, AdminAreaDbRow>(sql_poly)
        .bind(point.longitude())
        .bind(point.latitude())
        .fetch_all(pool)
        .await?;

    if !rows.is_empty() {
        let mut areas = Vec::with_capacity(rows.len());
        for row in rows {
            areas.push(AdministrativeArea::try_from(row)?);
        }

        let mut province = None;
        let mut county = None;
        let mut district = None;
        let mut city = None;
        let mut neighborhood = None;
        let mut country_code = "IRN".to_string();

        let mut breadcrumbs = Vec::new();
        let mut address_parts = Vec::new();

        for area in &areas {
            country_code = area.country_code.clone();
            breadcrumbs.push(AreaBreadcrumb {
                id: area.id,
                level: area.admin_level,
                area_type: area.area_type.clone(),
                name: area.name.clone(),
            });

            match area.area_type.as_str() {
                "neighborhood" => neighborhood = Some(area.name.clone()),
                "city" => city = Some(area.name.clone()),
                "district" => district = Some(area.name.clone()),
                "county" => county = Some(area.name.clone()),
                "province" => province = Some(area.name.clone()),
                _ => {}
            }
        }

        if let Some(ref p) = province { address_parts.push(p.as_str()); }
        if let Some(ref c) = city {
            if !address_parts.contains(&c.as_str()) { address_parts.push(c.as_str()); }
        }
        if let Some(ref n) = neighborhood { address_parts.push(n.as_str()); }

        let formatted_address = address_parts.join("، ");

        return Ok(Some(ReverseGeocodeResult {
            formatted_address,
            country_code,
            province,
            county,
            district,
            city,
            neighborhood,
            street: None,
            poi: None,
            hierarchy: breadcrumbs,
        }));
    }

    // ۲. اولویت دوم: استخراج هوشمند ۴ لایه (شهر، محله، خیابان، مکان عطف)

    // لایه الف: استخراج منحصراً شهر واقعی (place = city | town)
    let sql_city = r#"
        SELECT id, address_summary
        FROM locations
        WHERE metadata->>'place' IN ('city', 'town')
          AND address_summary IS NOT NULL
        ORDER BY coordinates <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)
        LIMIT 1
    "#;
    let city_row: Option<(uuid::Uuid, Option<String>)> = sqlx::query_as(sql_city)
        .bind(point.longitude())
        .bind(point.latitude())
        .fetch_optional(pool)
        .await?;

    // لایه ب: استخراج محله، منطقه یا میدان (place = suburb | neighbourhood | quarter | square)
    let sql_neighborhood = r#"
        SELECT id, address_summary
        FROM locations
        WHERE metadata->>'place' IN ('suburb', 'neighbourhood', 'quarter', 'square', 'locality')
          AND address_summary IS NOT NULL
        ORDER BY coordinates <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)
        LIMIT 1
    "#;
    let neighborhood_row: Option<(uuid::Uuid, Option<String>)> = sqlx::query_as(sql_neighborhood)
        .bind(point.longitude())
        .bind(point.latitude())
        .fetch_optional(pool)
        .await?;

    // لایه ج: استخراج نزدیک‌ترین خیابان از جدول معابر osm_roads
    let sql_street = r#"
        SELECT name
        FROM osm_roads
        WHERE name IS NOT NULL AND name != ''
        ORDER BY geom <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)
        LIMIT 1
    "#;
    let street_name: Option<String> = sqlx::query_scalar(sql_street)
        .bind(point.longitude())
        .bind(point.latitude())
        .fetch_optional(pool)
        .await?;

    // لایه د: استخراج مکان عطف یا فروشگاه (POI) در فاصله کمتر از ۲۰۰ متر
    let sql_poi = r#"
        SELECT address_summary,
               ST_Distance(coordinates::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
        FROM locations
        WHERE address_summary IS NOT NULL 
          AND (metadata->>'place' IS NULL OR metadata->>'place' NOT IN ('city', 'town', 'suburb', 'neighbourhood', 'square'))
        ORDER BY coordinates <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)
        LIMIT 1
    "#;
    let poi_row: Option<(Option<String>, f64)> = sqlx::query_as(sql_poi)
        .bind(point.longitude())
        .bind(point.latitude())
        .fetch_optional(pool)
        .await?;

    let city_name = city_row.and_then(|r| r.1);
    let neighborhood_name = neighborhood_row.and_then(|r| r.1);
    let poi_name = poi_row.and_then(|r| if r.1 < 250.0 { r.0 } else { None });

    let mut address_parts = Vec::new();
    let mut breadcrumbs = Vec::new();

    // ۱. درج شهر
    if let Some(ref c) = city_name {
        address_parts.push(c.clone());
        breadcrumbs.push(AreaBreadcrumb {
            id: uuid::Uuid::nil(),
            level: 8,
            area_type: "city".to_string(),
            name: c.clone(),
        });
    }

    // ۲. درج محله یا میدان
    if let Some(ref n) = neighborhood_name {
        if !address_parts.contains(n) {
            address_parts.push(n.clone());
            breadcrumbs.push(AreaBreadcrumb {
                id: uuid::Uuid::nil(),
                level: 10,
                area_type: "neighborhood".to_string(),
                name: n.clone(),
            });
        }
    }

    // ۳. درج خیابان
    if let Some(ref s) = street_name {
        if !address_parts.contains(s) {
            address_parts.push(s.clone());
        }
    }

    // ۴. درج مکان عطف (در صورت وجود در فاصله نزدیک)
    if let Some(ref p) = poi_name {
        if !address_parts.contains(p) {
            address_parts.push(format!("نزدیک {p}"));
        }
    }

    let formatted_address = if address_parts.is_empty() {
        "موقعیت در ایران".to_string()
    } else {
        address_parts.join("، ")
    };

    Ok(Some(ReverseGeocodeResult {
        formatted_address,
        country_code: "IRN".to_string(),
        province: None,
        county: None,
        district: None,
        city: city_name,
        neighborhood: neighborhood_name,
        street: street_name,
        poi: poi_name,
        hierarchy: breadcrumbs,
    }))
}