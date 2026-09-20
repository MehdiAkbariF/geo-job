use crate::admin_repo::AdminAreaDbRow;
use crate::error::GeocodingError;
use crate::models::{AdministrativeArea, AreaBreadcrumb, ReverseGeocodeResult};
use geo_types::GeoPoint;
use sqlx::PgPool;

pub async fn reverse_geocode_point(
    pool: &PgPool,
    point: &GeoPoint,
) -> Result<Option<ReverseGeocodeResult>, GeocodingError> {
    // Spatial point-in-polygon containment query.
    // Ordered from most specific (highest admin_level) to broadest.
    let sql = r#"
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

    let rows = sqlx::query_as::<_, AdminAreaDbRow>(sql)
        .bind(point.longitude())
        .bind(point.latitude())
        .fetch_all(pool)
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }

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

    // Areas are sorted by admin_level DESC (neighborhood -> city -> county -> province)
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

    // Build human-readable formatted address (e.g. "استان تهران، تهران، سعادت‌آباد")
    if let Some(ref p) = province {
        address_parts.push(p.as_str());
    }
    if let Some(ref c) = city {
        if !address_parts.contains(&c.as_str()) {
            address_parts.push(c.as_str());
        }
    }
    if let Some(ref n) = neighborhood {
        address_parts.push(n.as_str());
    }

    let formatted_address = address_parts.join("، ");

    Ok(Some(ReverseGeocodeResult {
        formatted_address,
        country_code,
        province,
        county,
        district,
        city,
        neighborhood,
        hierarchy: breadcrumbs,
    }))
}