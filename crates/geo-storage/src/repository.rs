use crate::error::StorageError;
use crate::models::LocationDbRow;
use geo_domain::{Location, NewLocation};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LocationRepository {
    pool: PgPool,
}

impl LocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Inserts a new geographic location into PostGIS with SRID 4326.
    pub async fn insert(&self, item: &NewLocation) -> Result<Location, StorageError> {
        let precision_str = match item.precision {
            geo_domain::LocationPrecision::Exact => "exact",
            geo_domain::LocationPrecision::Rooftop => "rooftop",
            geo_domain::LocationPrecision::Street => "street",
            geo_domain::LocationPrecision::Neighborhood => "neighborhood",
            geo_domain::LocationPrecision::City => "city",
            geo_domain::LocationPrecision::Approximate => "approximate",
        };

        let sql = r#"
            INSERT INTO locations (
                coordinates,
                address_summary,
                precision,
                source,
                source_id,
                metadata
            )
            VALUES (
                ST_SetSRID(ST_MakePoint($1, $2), 4326),
                $3,
                $4,
                $5,
                $6,
                $7
            )
            RETURNING 
                id,
                ST_X(coordinates::geometry) AS longitude,
                ST_Y(coordinates::geometry) AS latitude,
                address_summary,
                precision,
                source,
                source_id,
                metadata,
                created_at,
                updated_at
        "#;

        let row = sqlx::query_as::<_, LocationDbRow>(sql)
            .bind(item.point.longitude())
            .bind(item.point.latitude())
            .bind(&item.address_summary)
            .bind(precision_str)
            .bind(&item.source)
            .bind(&item.source_id)
            .bind(&item.metadata)
            .fetch_one(&self.pool)
            .await?;

        row.try_into()
    }

    /// Finds a single location by its primary identifier.
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Location>, StorageError> {
        let sql = r#"
            SELECT 
                id,
                ST_X(coordinates::geometry) AS longitude,
                ST_Y(coordinates::geometry) AS latitude,
                address_summary,
                precision,
                source,
                source_id,
                metadata,
                created_at,
                updated_at
            FROM locations
            WHERE id = $1
        "#;

        let row = sqlx::query_as::<_, LocationDbRow>(sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        row.map(Location::try_from).transpose()
    }
}