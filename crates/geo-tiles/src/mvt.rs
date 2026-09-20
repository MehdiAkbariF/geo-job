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

/// Ultra-stable MVT generator.
/// Enforces MVT-compliant LineString geometries using ST_CollectionExtract to prevent 'Unimplemented type: 4'.
pub async fn get_base_map_mvt_tile(
    pool: &PgPool,
    tile: &TileCoordinate,
) -> Result<Vec<u8>, MvtError> {
    let sql = r#"
        WITH bounds AS (
            SELECT 
                ST_TileEnvelope($1, $2, $3) AS geom_3857,
                ST_Transform(ST_TileEnvelope($1, $2, $3), 4326) AS geom_4326
        ),
        roads_mvt AS (
            SELECT COALESCE(ST_AsMVT(r_tile, 'roads', 4096, 'geom'), ''::bytea) AS mvt
            FROM (
                SELECT 
                    r.id,
                    r.name,
                    r.highway,
                    ST_CollectionExtract(
                        ST_AsMVTGeom(
                            ST_Transform(r.geom, 3857),
                            b.geom_3857,
                            4096,
                            256,
                            true
                        ),
                        2 -- Enforce pure LineString / MultiLineString (MVT Type 2)
                    ) AS geom
                FROM osm_roads r, bounds b
                WHERE r.geom && b.geom_4326
                  AND (
                      ($1 <= 8 AND r.highway IN ('motorway', 'trunk', 'primary', 'motorway_link', 'trunk_link'))
                      OR ($1 BETWEEN 9 AND 12 AND r.highway IN ('motorway', 'trunk', 'primary', 'secondary', 'tertiary', 'motorway_link', 'trunk_link', 'primary_link'))
                      OR ($1 >= 13)
                  )
            ) r_tile
            WHERE r_tile.geom IS NOT NULL AND NOT ST_IsEmpty(r_tile.geom)
        ),
        boundaries_mvt AS (
            SELECT COALESCE(ST_AsMVT(b_tile, 'boundaries', 4096, 'geom'), ''::bytea) AS mvt
            FROM (
                SELECT 
                    a.id,
                    a.name,
                    a.area_type,
                    ST_CollectionExtract(
                        ST_AsMVTGeom(
                            ST_Transform(ST_Boundary(a.boundary), 3857),
                            b.geom_3857,
                            4096,
                            256,
                            true
                        ),
                        2 -- Enforce pure LineString for boundaries
                    ) AS geom
                FROM administrative_areas a, bounds b
                WHERE a.boundary && b.geom_4326
            ) b_tile
            WHERE b_tile.geom IS NOT NULL AND NOT ST_IsEmpty(b_tile.geom)
        )
        SELECT (roads_mvt.mvt || boundaries_mvt.mvt) AS combined_mvt
        FROM roads_mvt, boundaries_mvt
    "#;

    let row: (Vec<u8>,) = sqlx::query_as(sql)
        .bind(tile.z as i32)
        .bind(tile.x as i32)
        .bind(tile.y as i32)
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}

/// Single layer MVT for employer opportunities.
pub async fn get_locations_mvt_tile(
    pool: &PgPool,
    tile: &TileCoordinate,
) -> Result<Vec<u8>, MvtError> {
    let sql = r#"
        WITH bounds AS (
            SELECT 
                ST_TileEnvelope($1, $2, $3) AS geom_3857,
                ST_Transform(ST_TileEnvelope($1, $2, $3), 4326) AS geom_4326
        ),
        mvtgeom AS (
            SELECT 
                l.id,
                l.address_summary,
                l.precision,
                ST_CollectionExtract(
                    ST_AsMVTGeom(
                        ST_Transform(l.coordinates, 3857),
                        b.geom_3857,
                        4096,
                        256,
                        true
                    ),
                    1 -- Enforce Point (MVT Type 1)
                ) AS geom
            FROM locations l, bounds b
            WHERE l.coordinates && b.geom_4326
              AND l.source = 'opportunity'
        )
        SELECT COALESCE(ST_AsMVT(mvtgeom.*, 'locations', 4096, 'geom'), ''::bytea) AS mvt
        FROM mvtgeom
        WHERE mvtgeom.geom IS NOT NULL AND NOT ST_IsEmpty(mvtgeom.geom)
    "#;

    let row: (Vec<u8>,) = sqlx::query_as(sql)
        .bind(tile.z as i32)
        .bind(tile.x as i32)
        .bind(tile.y as i32)
        .fetch_one(pool)
        .await?;

    Ok(row.0)
}