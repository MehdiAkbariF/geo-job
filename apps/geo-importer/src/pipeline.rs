use crate::error::ImporterError;
use crate::osm_parser::ExtractedOsmPoint;
use geo_types::GeoPoint;
use sqlx::PgPool;

#[allow(dead_code)]
pub async fn insert_osm_batch(
    pool: &PgPool,
    points: &[ExtractedOsmPoint],
) -> Result<u64, ImporterError> {
    if points.is_empty() {
        return Ok(0);
    }

    let mut tx = pool.begin().await?;

    for p in points {
        let metadata = serde_json::to_value(&p.tags).unwrap_or_else(|_| serde_json::json!({}));
        let source_id = p.osm_id.to_string();

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
                'rooftop',
                'osm',
                $4,
                $5
            )
            ON CONFLICT (source, source_id) 
            DO UPDATE SET
                address_summary = EXCLUDED.address_summary,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
        "#;

        sqlx::query(sql)
            .bind(p.point.longitude())
            .bind(p.point.latitude())
            .bind(&p.name)
            .bind(&source_id)
            .bind(&metadata)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(points.len() as u64)
}

/// Seeds standard opportunity and landmark locations in Tehran for immediate testing.
pub async fn seed_sample_locations(pool: &PgPool) -> Result<u64, ImporterError> {
    let samples = vec![
        ("دفتر مرکزی دیجی‌کالا", 51.4116, 35.7523, "administrative"),
        ("پارک فناوری پردیس", 51.8153, 35.7314, "tech_park"),
        ("میدان آزادی", 51.3381, 35.7006, "landmark"),
        ("برج میلاد", 51.3753, 35.7448, "landmark"),
        ("ایستگاه مترو صادقیه", 51.3212, 35.7219, "transit"),
        ("دانشگاه تهران", 51.3890, 35.7036, "education"),
        ("دانشگاه صنعتی شریف", 51.3516, 35.7036, "education"),
    ];

    for (name, lon, lat, cat) in samples {
        let pt = GeoPoint::new(lon, lat).map_err(|e| ImporterError::Validation(e.to_string()))?;
        let metadata = serde_json::json!({ "category": cat, "city": "Tehran" });

        sqlx::query(r#"
            INSERT INTO locations (coordinates, address_summary, precision, source, source_id, metadata)
            VALUES (ST_SetSRID(ST_MakePoint($1, $2), 4326), $3, 'exact', 'sample_seed', $4, $5)
            ON CONFLICT (source, source_id) DO UPDATE SET updated_at = NOW()
        "#)
        .bind(pt.longitude())
        .bind(pt.latitude())
        .bind(name)
        .bind(name)
        .bind(metadata)
        .execute(pool)
        .await?;
    }

    tracing::info!("Successfully seeded 7 sample landmark locations.");
    Ok(7)
}