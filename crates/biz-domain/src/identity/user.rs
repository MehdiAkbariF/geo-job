use crate::error::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Suspended,
    PendingVerification,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::PendingVerification => "pending_verification",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(Self::Active),
            "suspended" => Some(Self::Suspended),
            "pending_verification" => Some(Self::PendingVerification),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: Option<String>,
    pub user_type: String, // candidate, employer, project_client, admin
    pub national_id: Option<String>,
    pub is_phone_verified: bool,
    pub is_onboarded: bool,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: Option<String>,
}

impl User {
    /// اعتبارسنجی فرمت شماره موبایل ایرانی (شروع با 09 و 11 رقم)
    pub fn validate_iranian_phone(phone: &str) -> Result<String, DomainError> {
        let clean = phone.trim().replace("+98", "0");
        if clean.len() == 11 && clean.starts_with("09") && clean.chars().all(|c| c.is_ascii_digit()) {
            Ok(clean)
        } else {
            Err(DomainError::InvalidPhoneNumber("شماره موبایل وارد شده باید یک شماره معتبر ۱۱ رقمی ایرانی (مثلاً 09123456789) باشد".into()))
        }
    }

    /// اعتبارسنجی الگوریتم ۱۰ رقمی کد ملی ایران
    pub fn validate_national_id(code: &str) -> Result<String, DomainError> {
        let clean = code.trim();
        if clean.len() != 10 || !clean.chars().all(|c| c.is_ascii_digit()) {
            return Err(DomainError::InvariantViolation("کد ملی باید ۱۰ رقم عددی باشد".into()));
        }

        let digits: Vec<u32> = clean.chars().filter_map(|c| c.to_digit(10)).collect();
        let check_digit = digits[9];
        let sum: u32 = (0..9).map(|i| digits[i] * (10 - i as u32)).sum();
        let remainder = sum % 11;

        let is_valid = if remainder < 2 {
            check_digit == remainder
        } else {
            check_digit == (11 - remainder)
        };

        if is_valid {
            Ok(clean.to_string())
        } else {
            Err(DomainError::InvariantViolation("کد ملی وارد شده طبق الگوریتم ثبت احوال معتبر نمی‌باشد".into()))
        }
    }
}