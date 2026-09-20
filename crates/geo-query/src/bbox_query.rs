use crate::error::QueryError;
use geo_domain::Location;
use geo_storage::models::LocationDbRow;
use geo_types::BoundingBox;
use sqlx::PgPool;

const MAX_BBOX_LIMIT: usize = 1000;
const DEFAULT_BBOX_LIMIT: usize = 200;

#[derive(Debug, Clone)]
pub struct BBoxSearchOptions {
    pub limit: Option<usize>,
    pub source: Option<String>,
}

pub async fn find_locations_in_bbox(
    pool: &PgPool,
    bbox: &BoundingBox,
    options: BBoxSearchOptions,
) -> Result<Vec<Location>, QueryError> {
    let limit = options.limit.unwrap_or(DEFAULT_BBOX_LIMIT);
    if limit > MAX_BBOX_LIMIT {
        return Err(QueryError::LimitExceeded {
            requested: limit,
            max: MAX_BBOX_LIMIT,
        });
    }

    // Filters out background OSM data by default so only employer-posted opportunities become markers.
    let sql = r#"
        SELECT 
            id,
            ST_X(coordinates::geometry) AS longitude,
            ST_Y(coordinates::geometry) AS latitude,
            address_summary,
            precision,
            source,
            source_id,
            metadata,
            created_at,
            updated_at
        FROM locations
        WHERE coordinates && ST_MakeEnvelope($1, $2, $3, $4, 4326)
          AND (
              ($6::text IS NOT NULL AND source = $6)
              OR ($6::text IS NULL AND source != 'osm')
          )
        LIMIT $5
    "#;

    let rows = sqlx::query_as::<_, LocationDbRow>(sql)
        .bind(bbox.west())
        .bind(bbox.south())
        .bind(bbox.east())
        .bind(bbox.north())
        .bind(limit as i64)
        .bind(options.source)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(|row| Location::try_from(row).map_err(QueryError::from))
        .collect()
}