use crate::error::QueryError;
use crate::models::NearbyLocation;
use geo_domain::Location;
use geo_storage::models::LocationDbRow;
use geo_types::{Distance, GeoPoint, Radius};
use sqlx::PgPool;

const MAX_RADIUS_LIMIT: usize = 500;
const DEFAULT_RADIUS_LIMIT: usize = 100;

#[derive(Debug, sqlx::FromRow)]
struct NearbyDbRow {
    #[sqlx(flatten)]
    location: LocationDbRow,
    distance_meters: f64,
}

pub async fn find_locations_within_radius(
    pool: &PgPool,
    center: &GeoPoint,
    radius: &Radius,
    limit: Option<usize>,
) -> Result<Vec<NearbyLocation>, QueryError> {
    let limit = limit.unwrap_or(DEFAULT_RADIUS_LIMIT);
    if limit > MAX_RADIUS_LIMIT {
        return Err(QueryError::LimitExceeded {
            requested: limit,
            max: MAX_RADIUS_LIMIT,
        });
    }

    // Uses ST_DWithin on geography type for accurate geodesic distance in meters.
    // GiST index on geometry column supports geography cast optimization in PostGIS.
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
            updated_at,
            ST_Distance(
                coordinates::geography, 
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography
            ) AS distance_meters
        FROM locations
        WHERE ST_DWithin(
            coordinates::geography,
            ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
            $3
        )
        ORDER BY distance_meters ASC
        LIMIT $4
    "#;

    let rows = sqlx::query_as::<_, NearbyDbRow>(sql)
        .bind(center.longitude())
        .bind(center.latitude())
        .bind(radius.as_meters())
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(|row| {
            let location = Location::try_from(row.location)?;
            let distance = Distance::from_meters(row.distance_meters);
            Ok(NearbyLocation { location, distance })
        })
        .collect()
}

/// Finds the k-nearest locations using PostGIS GiST distance operator `<->`.
pub async fn find_nearest_neighbors(
    pool: &PgPool,
    center: &GeoPoint,
    limit: usize,
) -> Result<Vec<NearbyLocation>, QueryError> {
    let limit = limit.min(MAX_RADIUS_LIMIT);

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
            updated_at,
            ST_Distance(
                coordinates::geography, 
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography
            ) AS distance_meters
        FROM locations
        ORDER BY coordinates <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)
        LIMIT $3
    "#;

    let rows = sqlx::query_as::<_, NearbyDbRow>(sql)
        .bind(center.longitude())
        .bind(center.latitude())
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

    rows.into_iter()
        .map(|row| {
            let location = Location::try_from(row.location)?;
            let distance = Distance::from_meters(row.distance_meters);
            Ok(NearbyLocation { location, distance })
        })
        .collect()
}