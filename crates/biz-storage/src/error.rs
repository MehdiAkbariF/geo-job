use biz_domain::DomainError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Phone number already exists")]
    PhoneAlreadyExists,

    #[error("Candidate has already applied for this opportunity")]
    DuplicateApplication,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Opportunity not found")]
    OpportunityNotFound,

    #[error("Company not found")]
    CompanyNotFound,

    #[error("Candidate profile not found")]
    CandidateNotFound,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Security error: {0}")]
    Security(String),
}