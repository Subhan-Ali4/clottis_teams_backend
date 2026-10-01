use actix_web::{HttpResponse, web};

use crate::{
    executors::{execute_liveness, execute_readiness},
    structs::{ApiResponse, AppState},
    utils::AppError,
};

pub fn configure(config: &mut web::ServiceConfig) {
    config
        .route("/health", web::get().to(liveness))
        .route("/health/ready", web::get().to(readiness));
}

async fn liveness(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::success(execute_liveness(state.get_ref())))
}

async fn readiness(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let data = execute_readiness(state.get_ref()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(data)))
}

#[cfg(test)]
mod tests {
    use actix_web::{App, test, web};
    use sqlx::postgres::PgPoolOptions;

    use super::*;
    use crate::configs::{AppConfig, Environment};

    #[actix_rt::test]
    async fn liveness_returns_structured_success_response() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/clottis_teams")
            .unwrap();
        let state = web::Data::new(AppState::new(
            pool,
            AppConfig {
                app_name: "test-service".to_owned(),
                environment: Environment::Test,
                host: "127.0.0.1".to_owned(),
                port: 0,
                database_url: "postgres://postgres:postgres@localhost/clottis_teams".to_owned(),
                database_max_connections: 1,
                cors_allowed_origin: "http://localhost:3000".to_owned(),
            },
        ));

        let app = test::init_service(
            App::new()
                .app_data(state)
                .route("/health", web::get().to(liveness)),
        )
        .await;
        let request = test::TestRequest::get().uri("/health").to_request();
        let response: serde_json::Value = test::call_and_read_body_json(&app, request).await;

        assert_eq!(response["success"], true);
        assert_eq!(response["data"]["service"], "test-service");
        assert_eq!(response["data"]["status"], "healthy");
    }
}
