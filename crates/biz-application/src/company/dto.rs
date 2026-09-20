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
pub struct CompanyMemberDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: String,
}