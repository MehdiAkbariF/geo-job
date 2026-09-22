use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateCompanyCommand {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub website: Option<String>,
}

/// Comprehensive Employer Onboarding Command (Company + Legal Verification Evidence)
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct OnboardCompanyCommand {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub registration_number: String,
    pub national_id: String,
    pub license_storage_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateCompanyCommand {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub logo_storage_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddCompanyLocationCommand {
    #[schema(value_type = String, format = Uuid)]
    pub location_id: Uuid,
    pub is_headquarters: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompanyLocationDto {
    pub location_id: Uuid,
    pub address_summary: Option<String>,
    #[schema(value_type = [f64; 2], example = json!([51.4172, 35.7592]))]
    pub coordinates: [f64; 2],
    pub is_headquarters: bool,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct AddMemberCommand {
    #[schema(value_type = String, format = Uuid)]
    pub user_id: Uuid,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompanyDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_storage_key: Option<String>,
    pub website: Option<String>,
    pub verification_status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct UserCompanyMembershipDto {
    pub company: CompanyDto,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompanyMemberDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}