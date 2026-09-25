use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use biz_application::ApplicationError;
use biz_domain::DomainError;
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

        tracing::error!(request_id = %request_id, error = ?self, "API Error encountered");

        let (status, code, message) = match self {
            ApiError::Application(ApplicationError::Validation(msg))
            | ApiError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            
            ApiError::Application(ApplicationError::Domain(DomainError::InvariantViolation(msg))) => {
                (StatusCode::BAD_REQUEST, "INVALID_STATE_TRANSITION", msg)
            }

            ApiError::Application(ApplicationError::Unauthorized(msg))
            | ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
            
            ApiError::Application(ApplicationError::Storage(StorageError::InvalidCredentials))
            | ApiError::Storage(StorageError::InvalidCredentials) => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                "پست الکترونیک یا کلمه عبور وارد شده نادرست است".to_string(),
            ),
            
            ApiError::Application(ApplicationError::Storage(StorageError::EmailAlreadyExists))
            | ApiError::Storage(StorageError::EmailAlreadyExists) => (
                StatusCode::CONFLICT,
                "EMAIL_ALREADY_EXISTS",
                "کاربری با این ایمیل قبلاً در سیستم ثبت‌نام کرده است".to_string(),
            ),

            ApiError::Application(ApplicationError::Storage(StorageError::PhoneAlreadyExists))
            | ApiError::Storage(StorageError::PhoneAlreadyExists) => (
                StatusCode::CONFLICT,
                "PHONE_ALREADY_EXISTS",
                "کاربری با این شماره همراه قبلاً در سیستم ثبت‌نام کرده است".to_string(),
            ),
            
            ApiError::Application(ApplicationError::Storage(StorageError::DuplicateApplication))
            | ApiError::Storage(StorageError::DuplicateApplication) => (
                StatusCode::CONFLICT,
                "DUPLICATE_APPLICATION",
                "شما قبلاً برای این موقعیت شغلی رزومه ارسال کرده‌اید".to_string(),
            ),
            
            ApiError::Application(ApplicationError::Storage(StorageError::UserNotFound))
            | ApiError::Storage(StorageError::UserNotFound) => (
                StatusCode::NOT_FOUND,
                "USER_NOT_FOUND",
                "کاربر مورد نظر یافت نشد".to_string(),
            ),

            ApiError::Application(ApplicationError::Storage(StorageError::OpportunityNotFound))
            | ApiError::Storage(StorageError::OpportunityNotFound) => (
                StatusCode::NOT_FOUND,
                "OPPORTUNITY_NOT_FOUND",
                "فرصت شغلی مورد نظر یافت نشد یا منقضی شده است".to_string(),
            ),

            ApiError::Application(ApplicationError::Storage(StorageError::CompanyNotFound))
            | ApiError::Storage(StorageError::CompanyNotFound) => (
                StatusCode::NOT_FOUND,
                "COMPANY_NOT_FOUND",
                "سازمان یا شرکت مورد نظر یافت نشد".to_string(),
            ),

            ApiError::Application(ApplicationError::Storage(StorageError::CandidateNotFound))
            | ApiError::Storage(StorageError::CandidateNotFound) => (
                StatusCode::NOT_FOUND,
                "CANDIDATE_NOT_FOUND",
                "پروفایل کارجو یافت نشد".to_string(),
            ),
            
            err => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                format!("یک خطای غیرمنتظره در سرور رخ داد: {}", err),
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