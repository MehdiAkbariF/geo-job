use biz_domain::company::Company;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SendOtpCommand {
    pub phone: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct VerifyOtpCommand {
    pub phone: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OtpAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_secs: u64,
    pub user_id: Uuid,
    pub phone: Option<String>,
    pub is_new_user: bool,
    pub is_onboarded: bool,
    pub user_type: String,
}

// آنبوردینگ ۳ پرسونای هویتی
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CandidateOnboardingDto {
    pub first_name: String,
    pub last_name: String,
    pub preferred_city: String,
    pub headline: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct EmployerOnboardingDto {
    pub business_name: String,
    pub slug: String,
    pub business_type: String, // corporate, retail_shop, restaurant_cafe, clinic_office, workshop
    pub trade_license_number: Option<String>,
    pub location_id: Option<Uuid>,
}

/// فرم آنبوردینگ کارفرمای پروژه‌ای (شخصی و بدون نیاز به مغازه یا لوکیشن روی نقشه)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ProjectClientOnboardingDto {
    pub client_name: String,
    pub field_of_activity: String, // مثلاً: برنامه‌نویسی و IT، معماری، ترجمه، بازاریابی
    pub city: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct OnboardingCommand {
    pub role: String, // candidate, employer, project_client
    pub national_id: Option<String>,
    pub candidate_info: Option<CandidateOnboardingDto>,
    pub employer_info: Option<EmployerOnboardingDto>,
    pub project_client_info: Option<ProjectClientOnboardingDto>,
}

// خروجی جامع و غنی اطلاعات کاربر فعلی (GET /me)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserContextDto {
    pub user_id: Uuid,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub user_type: String,
    pub is_onboarded: bool,
    pub is_phone_verified: bool,
    pub candidate_id: Option<Uuid>,
    pub candidate_name: Option<String>,
    pub candidate_city: Option<String>,
    pub active_company_id: Option<Uuid>,
    pub active_company_name: Option<String>,
    pub active_company_type: Option<String>,
    pub active_company_role: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RegisterCommand {
    pub email: String,
    pub password: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LoginCommand {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct RefreshTokenCommand {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct LogoutCommand {
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_secs: u64,
    pub user_id: Uuid,
    pub email: Option<String>,
}

pub type AuthResponseDto = AuthResponse;