use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AuthResponseDto {
    pub user_id: Uuid,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in_secs: u64,
}