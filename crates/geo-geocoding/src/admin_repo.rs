use crate::error::GeocodingError;
use crate::models::AdministrativeArea;
use geo_types::GeoPoint;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct AdminAreaDbRow {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub country_code: String,
    pub admin_level: i32,
    pub area_type: String,
    pub name: String,
    pub name_en: Option<String>,
    pub center_lon: Option<f64>,
    pub center_lat: Option<f64>,
    pub metadata: serde_json::Value,
}

impl TryFrom<AdminAreaDbRow> for AdministrativeArea {
    type Error = GeocodingError;

    fn try_from(row: AdminAreaDbRow) -> Result<Self, Self::Error> {
        let center = match (row.center_lon, row.center_lat) {
            (Some(lon), Some(lat)) => Some(GeoPoint::new(lon, lat)?),
            _ => None,
        };

        Ok(Self {
            id: row.id,
            parent_id: row.parent_id,
            country_code: row.country_code,
            admin_level: row.admin_level,
            area_type: row.area_type,
            name: row.name,
            name_en: row.name_en,
            center,
            metadata: row.metadata,
        })
    }
}

pub async fn find_areas_by_parent(
    pool: &PgPool,
    parent_id: Option<Uuid>,
) -> Result<Vec<AdministrativeArea>, GeocodingError> {
    let sql = r#"
        SELECT 
            id,
            parent_id,
            country_code,
            admin_level,
            area_type,
            name,
            name_en,
            ST_X(center::geometry) AS center_lon,
            ST_Y(center::geometry) AS center_lat,
            metadata
        FROM administrative_areas
        WHERE ($1::uuid IS NULL AND parent_id IS NULL) OR parent_id = $1
        ORDER BY name ASC
    "#;

    let rows = sqlx::query_as::<_, AdminAreaDbRow>(sql)
        .bind(parent_id)
        .fetch_all(pool)
        .await?;

    rows.into_iter().map(AdministrativeArea::try_from).collect()
}