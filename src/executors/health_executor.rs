use chrono::Utc;

use crate::{
    services::{DatabaseHealth, check_database},
    structs::{AppState, DependencyStatus, HealthData, ReadinessData, ServiceStatus},
    utils::AppError,
};

pub fn execute_liveness(state: &AppState) -> HealthData {
    HealthData {
        service: state.config.app_name.clone(),
        status: ServiceStatus::Healthy,
        environment: state.config.environment,
        version: env!("CARGO_PKG_VERSION"),
        timestamp: Utc::now(),
    }
}

pub async fn execute_readiness(state: &AppState) -> Result<ReadinessData, AppError> {
    let database = match check_database(&state.db).await? {
        DatabaseHealth::Connected => DependencyStatus::Connected,
    };

    Ok(ReadinessData {
        service: state.config.app_name.clone(),
        status: ServiceStatus::Ready,
        database,
        timestamp: Utc::now(),
    })
}
