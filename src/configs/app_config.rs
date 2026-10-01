use std::{env, str::FromStr};

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_name: String,
    pub environment: Environment,
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub cors_allowed_origin: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            app_name: env_or("APP_NAME", "clottis-teams-backend"),
            environment: env_or("APP_ENV", "development").parse()?,
            host: env_or("APP_HOST", "0.0.0.0"),
            port: parse_env("APP_PORT", 8080)?,
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            database_max_connections: parse_env("DATABASE_MAX_CONNECTIONS", 10)?,
            cors_allowed_origin: env_or("CORS_ALLOWED_ORIGIN", "http://localhost:3000"),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl FromStr for Environment {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "development" | "dev" => Ok(Self::Development),
            "test" => Ok(Self::Test),
            "production" | "prod" => Ok(Self::Production),
            _ => Err(ConfigError::InvalidValue {
                key: "APP_ENV",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
    #[error("invalid value for {key}: {value}")]
    InvalidValue { key: &'static str, value: String },
}

fn env_or(key: &'static str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn parse_env<T>(key: &'static str, default: T) -> Result<T, ConfigError>
where
    T: FromStr + ToString,
{
    let value = env::var(key).unwrap_or_else(|_| default.to_string());
    value
        .parse()
        .map_err(|_| ConfigError::InvalidValue { key, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_environment_names() {
        assert_eq!(
            "dev".parse::<Environment>().unwrap(),
            Environment::Development
        );
        assert_eq!("test".parse::<Environment>().unwrap(), Environment::Test);
        assert_eq!(
            "prod".parse::<Environment>().unwrap(),
            Environment::Production
        );
    }
}
