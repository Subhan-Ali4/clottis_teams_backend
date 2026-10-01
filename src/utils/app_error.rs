use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use thiserror::Error;
use tracing::error;

use crate::structs::ErrorResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database operation failed")]
    Database(#[source] sqlx::Error),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_response(&self) -> HttpResponse {
        error!(error = ?self, "request failed");
        HttpResponse::build(self.status_code()).json(ErrorResponse::new(
            "service_unavailable",
            "A required service is temporarily unavailable.",
        ))
    }
}
