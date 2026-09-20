use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use biz_application::ApplicationError;
use biz_storage::StorageError;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ApiErrorPayload {
    pub code: String,
    pub message: String,
    pub details: serde_json::Value,
    pub request_id: String,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorPayload,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Application error: {0}")]
    Application(#[from] ApplicationError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation failed: {0}")]
    Validation(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let request_id = Uuid::new_v4().to_string();
        let (status, code, message) = match self {
            ApiError::Application(ApplicationError::Validation(msg))
            | ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            
            ApiError::Application(ApplicationError::Unauthorized(msg))
            | ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            
            ApiError::Storage(StorageError::InvalidCredentials) => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "Email or password is incorrect".to_string(),
            ),
            
            ApiError::Storage(StorageError::EmailAlreadyExists) => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                "A user with this email already exists".to_string(),
            ),
            
            ApiError::Storage(StorageError::DuplicateApplication) => (
                StatusCode::CONFLICT,
                "DUPLICATE_APPLICATION",
                "You have already applied for this opportunity".to_string(),
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
                details: serde_json::json!({}),
                request_id,
            },
        };

        (status, Json(payload)).into_response()
    }
}