use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use geo_tiles::{get_base_map_mvt_tile, get_locations_mvt_tile, TileCoordinate};

/// Serves multi-layer Base Map Vector Tiles (Roads, Boundaries, Places) from PostGIS.
pub async fn serve_base_map_tile(
    State(state): State<AppState>,
    Path((z, x, tile_str)): Path<(u8, u32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let y_str = tile_str
        .strip_suffix(".mvt")
        .or_else(|| tile_str.strip_suffix(".pbf"))
        .unwrap_or(&tile_str);

    let y: u32 = y_str
        .parse()
        .map_err(|_| ApiError::Validation(format!("Invalid tile Y coordinate: {tile_str}")))?;

    let tile = TileCoordinate::new(z, x, y)?;
    let pbf_bytes = get_base_map_mvt_tile(&state.pool, &tile)
        .await
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/x-protobuf".parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());

    Ok((StatusCode::OK, headers, pbf_bytes))
}

/// Serves dynamic Locations layer MVT.
pub async fn serve_vector_tile(
    State(state): State<AppState>,
    Path((z, x, tile_str)): Path<(u8, u32, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let y_str = tile_str
        .strip_suffix(".mvt")
        .or_else(|| tile_str.strip_suffix(".pbf"))
        .unwrap_or(&tile_str);

    let y: u32 = y_str
        .parse()
        .map_err(|_| ApiError::Validation(format!("Invalid tile Y coordinate: {tile_str}")))?;

    let tile = TileCoordinate::new(z, x, y)?;
    let pbf_bytes = get_locations_mvt_tile(&state.pool, &tile)
        .await
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/x-protobuf".parse().unwrap());
    headers.insert(header::CACHE_CONTROL, "public, max-age=300".parse().unwrap());

    Ok((StatusCode::OK, headers, pbf_bytes))
}