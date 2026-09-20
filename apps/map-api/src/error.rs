use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use geo_query::QueryError;
use geo_storage::StorageError;
use geo_tiles::TileError;
use geo_types::GeoError;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorPayload,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Geospatial validation failed: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Spatial query failed: {0}")]
    Query(#[from] QueryError),

    #[error("Tile error: {0}")]
    Tile(#[from] TileError),

    #[error("Internal database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "INVALID_INPUT", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            ApiError::Storage(StorageError::NotFound(id)) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("Resource with id {id} was not found"),
            ),
            ApiError::Storage(StorageError::Domain(geo_err)) => (
                StatusCode::BAD_REQUEST,
                "INVALID_COORDINATES",
                geo_err.to_string(),
            ),
            ApiError::Query(QueryError::LimitExceeded { requested, max }) => (
                StatusCode::BAD_REQUEST,
                "LIMIT_EXCEEDED",
                format!("Requested limit {requested} exceeds maximum allowable limit of {max}"),
            ),
            ApiError::Query(QueryError::Domain(GeoError::InvalidBoundingBoxLatitude { .. }))
            | ApiError::Query(QueryError::Domain(GeoError::InvalidBoundingBoxLongitude { .. })) => (
                StatusCode::BAD_REQUEST,
                "INVALID_BBOX",
                "Bounding box coordinates are invalid or inverted".to_string(),
            ),
            ApiError::Tile(TileError::InvalidZoom(z)) => (
                StatusCode::BAD_REQUEST,
                "INVALID_ZOOM",
                format!("Zoom level {z} is outside valid range (0-22)"),
            ),
            ApiError::Tile(TileError::TileXOutOfBounds { .. })
            | ApiError::Tile(TileError::TileYOutOfBounds { .. }) => (
                StatusCode::BAD_REQUEST,
                "TILE_OUT_OF_BOUNDS",
                "Requested tile coordinate is out of bounds for the given zoom level".to_string(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "An unexpected internal error occurred".to_string(),
            ),
        };

        let payload = ApiErrorResponse {
            error: ApiErrorPayload {
                code: code.to_string(),
                message,
            },
        };

        (status, Json(payload)).into_response()
    }
}