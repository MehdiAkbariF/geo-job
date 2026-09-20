use geo_storage::LocationRepository;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub location_repo: LocationRepository,
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let location_repo = LocationRepository::new(pool.clone());
        Self { pool, location_repo }
    }
}