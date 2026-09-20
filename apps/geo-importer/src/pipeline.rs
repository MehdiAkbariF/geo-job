use crate::error::ImporterError;
use crate::osm_parser::{ExtractedOsmPoint, ExtractedOsmRoad};
use sqlx::PgPool;

/// High-performance bulk insert for real OSM road centerlines
pub async fn insert_roads_batch(
    pool: &PgPool,
    roads: &[ExtractedOsmRoad],
) -> Result<u64, ImporterError> {
    if roads.is_empty() {
        return Ok(0);
    }

    let mut ids = Vec::with_capacity(roads.len());
    let mut names = Vec::with_capacity(roads.len());
    let mut highways = Vec::with_capacity(roads.len());
    let mut wkts = Vec::with_capacity(roads.len());

    for r in roads {
        ids.push(r.osm_id);
        names.push(r.name.clone());
        highways.push(r.highway.clone());
        wkts.push(r.line_wkt.clone());
    }

    let sql = r#"
        INSERT INTO osm_roads (id, name, highway, geom)
        SELECT 
            u.id,
            u.name,
            u.highway,
            ST_SetSRID(ST_GeomFromText(u.wkt), 4326)
        FROM UNNEST(
            $1::bigint[],
            $2::text[],
            $3::text[],
            $4::text[]
        ) AS u(id, name, highway, wkt)
        ON CONFLICT (id) DO NOTHING
    "#;

    sqlx::query(sql)
        .bind(&ids)
        .bind(&names)
        .bind(&highways)
        .bind(&wkts)
        .execute(pool)
        .await?;

    Ok(roads.len() as u64)
}

/// Bulk insert for points/amenities/places
pub async fn insert_osm_batch(
    pool: &PgPool,
    points: &[ExtractedOsmPoint],
) -> Result<u64, ImporterError> {
    if points.is_empty() {
        return Ok(0);
    }

    let mut lons = Vec::with_capacity(points.len());
    let mut lats = Vec::with_capacity(points.len());
    let mut names = Vec::with_capacity(points.len());
    let mut source_ids = Vec::with_capacity(points.len());
    let mut metadatas = Vec::with_capacity(points.len());

    for p in points {
        lons.push(p.point.longitude());
        lats.push(p.point.latitude());
        names.push(p.name.clone());
        source_ids.push(p.osm_id.to_string());
        metadatas.push(serde_json::to_value(&p.tags).unwrap_or_else(|_| serde_json::json!({})));
    }

    let sql = r#"
        INSERT INTO locations (
            coordinates,
            address_summary,
            precision,
            source,
            source_id,
            metadata
        )
        SELECT 
            ST_SetSRID(ST_MakePoint(u.lon, u.lat), 4326),
            u.name,
            'rooftop',
            'osm',
            u.source_id,
            u.meta
        FROM UNNEST(
            $1::float8[],
            $2::float8[],
            $3::text[],
            $4::text[],
            $5::jsonb[]
        ) AS u(lon, lat, name, source_id, meta)
        ON CONFLICT (source, source_id) 
        DO UPDATE SET
            address_summary = EXCLUDED.address_summary,
            metadata = EXCLUDED.metadata,
            updated_at = NOW()
    "#;

    sqlx::query(sql)
        .bind(&lons)
        .bind(&lats)
        .bind(&names)
        .bind(&source_ids)
        .bind(&metadatas)
        .execute(pool)
        .await?;

    Ok(points.len() as u64)
}