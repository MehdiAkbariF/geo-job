use crate::error::ApiError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use geo_tiles::{get_locations_mvt_tile, TileCoordinate};

pub async fn serve_vector_tile(
    State(state): State<AppState>,
    Path((z, x, y)): Path<(u8, u32, u32)>,
) -> Result<impl IntoResponse, ApiError> {
    let tile = TileCoordinate::new(z, x, y)?;
    let pbf_bytes = get_locations_mvt_tile(&state.pool, &tile)
        .await
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/x-protobuf".parse().unwrap(),
    );
    headers.insert(
        header::CACHE_CONTROL,
        "public, max-age=300".parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, pbf_bytes))
}