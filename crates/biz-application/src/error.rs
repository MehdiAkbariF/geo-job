use biz_domain::DomainError;
use biz_storage::StorageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation failed: {0}")]
    Validation(String),
}