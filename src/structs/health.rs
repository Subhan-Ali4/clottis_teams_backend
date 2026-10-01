use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    Healthy,
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyStatus {
    Connected,
}

#[derive(Debug, Serialize)]
pub struct HealthData {
    pub service: String,
    pub status: ServiceStatus,
    pub environment: crate::configs::Environment,
    pub version: &'static str,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ReadinessData {
    pub service: String,
    pub status: ServiceStatus,
    pub database: DependencyStatus,
    pub timestamp: DateTime<Utc>,
}
