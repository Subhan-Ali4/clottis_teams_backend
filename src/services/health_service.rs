use sqlx::PgPool;

use crate::utils::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseHealth {
    Connected,
}

pub async fn check_database(pool: &PgPool) -> Result<DatabaseHealth, AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

    Ok(DatabaseHealth::Connected)
}
