use geo_types::GeoPoint;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdministrativeArea {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub country_code: String,
    pub admin_level: i32,
    pub area_type: String,
    pub name: String,
    pub name_en: Option<String>,
    pub center: Option<GeoPoint>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AreaBreadcrumb {
    pub id: Uuid,
    pub level: i32,
    pub area_type: String,
    pub name: String,
}

/// Fully structured 4-tier Reverse Geocoding model (City, Neighborhood, Street, POI)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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