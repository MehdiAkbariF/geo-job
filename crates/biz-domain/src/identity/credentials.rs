use crate::error::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: &str) -> Result<Self, DomainError> {
        let trimmed = email.trim().to_lowercase();
        if trimmed.is_empty() || !trimmed.contains('@') || !trimmed.contains('.') {
            return Err(DomainError::InvalidEmail(email.to_string()));
        }
        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPassword(String);

impl RawPassword {
    pub fn new(password: &str) -> Result<Self, DomainError> {
        if password.len() < 8 {
            return Err(DomainError::WeakPassword);
        }
        Ok(Self(password.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}