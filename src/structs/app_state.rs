use sqlx::PgPool;

use crate::configs::AppConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: AppConfig,
}

impl AppState {
    pub fn new(db: PgPool, config: AppConfig) -> Self {
        Self { db, config }
    }
}
