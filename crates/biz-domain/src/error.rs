use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum DomainError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Password is too weak: must be at least 8 characters.")]
    WeakPassword,

    #[error("Invalid phone number: {0}")]
    InvalidPhoneNumber(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Business invariant violated: {0}")]
    InvariantViolation(String),
}