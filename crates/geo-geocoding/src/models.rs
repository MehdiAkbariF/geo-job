use geo_types::GeoPoint;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AdministrativeArea {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub country_code: String,
    pub admin_level: i32,
    pub area_type: String,
    pub name: String,
    pub name_en: Option<String>,
    #[schema(value_type = Option<[f64; 2]>)]
    pub center: Option<GeoPoint>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct AreaBreadcrumb {
    pub id: Uuid,
    pub level: i32,
    pub area_type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ReverseGeocodeResult {
    pub formatted_address: String,
    pub country_code: String,
    pub province: Option<String>,
    pub county: Option<String>,
    pub district: Option<String>,
    pub city: Option<String>,
    pub neighborhood: Option<String>,
    pub street: Option<String>,
    pub poi: Option<String>,
    pub hierarchy: Vec<AreaBreadcrumb>,
}