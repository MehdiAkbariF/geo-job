use crate::tile_coord::{TileCoordinate, TileError};
use sqlx::PgPool;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MvtError {
    #[error("Tile error: {0}")]
    Tile(#[from] TileError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Generates a Mapbox Vector Tile (MVT) binary payload directly within PostGIS.
/// Uses `ST_TileEnvelope` and `ST_AsMVT` for sub-millisecond tile serialization.
pub async fn get_locations_mvt_tile(
    pool: &PgPool,
    tile: &TileCoordinate,
) -> Result<Vec<u8>, MvtError> {
    // ST_TileEnvelope creates the Web Mercator bounding box for tile (z, x, y).
    // ST_Transform transforms 4326 geometries into 3857 for tile projection.
    // ST_AsMVTGeom maps geometries to the 4096-unit tile extent.
    // ST_AsMVT compresses the entire layer into standard Protocol Buffer bytes (MVT).
    let sql = r#"
        WITH bounds AS (
            SELECT ST_TileEnvelope($1, $2, $3) AS geom
        ),
        mvtgeom AS (
            SELECT 
                l.id,
                l.address_summary,
                l.precision,
                ST_AsMVTGeom(
                    ST_Transform(l.coordinates, 3857),
                    b.geom,
                    4096,
                    256,
                    true
                ) AS geom
            FROM locations l, bounds b
            WHERE ST_Transform(l.coordinates, 3857) && b.geom
        )
        SELECT COALESCE(ST_AsMVT(mvtgeom.*, 'locations', 4096, 'geom'), ''::bytea) AS mvt
        FROM mvtgeom
    "#;

    let row: (Vec<u8>,) = sqlx::query_as(sql)
        .bind(tile.z as i32)
        .bind(tile.x as i32)
        .bind(tile.y as i32)
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}