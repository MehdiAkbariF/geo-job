use crate::error::QueryError;
use crate::models::LocationCluster;
use geo_types::{BoundingBox, GeoPoint};
use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
struct ClusterDbRow {
    count: i64,
    center_lon: f64,
    center_lat: f64,
    west: f64,
    south: f64,
    east: f64,
    north: f64,
}

/// Aggregates only employer opportunities within a bounding box.
/// OSM background points are strictly excluded.
pub async fn cluster_locations_in_bbox(
    pool: &PgPool,
    bbox: &BoundingBox,
    grid_size: f64,
) -> Result<Vec<LocationCluster>, QueryError> {
    let sql = r#"
        SELECT 
            COUNT(*) AS count,
            ST_X(ST_Centroid(ST_Collect(coordinates::geometry))) AS center_lon,
            ST_Y(ST_Centroid(ST_Collect(coordinates::geometry))) AS center_lat,
            ST_XMin(ST_Extent(coordinates::geometry)) AS west,
            ST_YMin(ST_Extent(coordinates::geometry)) AS south,
            ST_XMax(ST_Extent(coordinates::geometry)) AS east,
            ST_YMax(ST_Extent(coordinates::geometry)) AS north
        FROM locations
        WHERE coordinates && ST_MakeEnvelope($1, $2, $3, $4, 4326)
          AND source = 'opportunity'
        GROUP BY ST_SnapToGrid(coordinates, $5)
        HAVING COUNT(*) > 1
    "#;

    let rows = sqlx::query_as::<_, ClusterDbRow>(sql)
        .bind(bbox.west())
        .bind(bbox.south())
        .bind(bbox.east())
        .bind(bbox.north())
        .bind(grid_size)
        .fetch_all(pool)
        .await?;

    let mut clusters = Vec::with_capacity(rows.len());
    for row in rows {
        let center = GeoPoint::new(row.center_lon, row.center_lat)?;
        let bounds = BoundingBox::new(row.west, row.south, row.east, row.north)?;
        clusters.push(LocationCluster {
            count: row.count,
            center,
            bounds,
        });
    }

    Ok(clusters)
}