use std::io;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use clottis_teams_backend::{configs::AppConfig, routes, structs::AppState};
use sqlx::postgres::PgPoolOptions;
use tracing::info;

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env().map_err(io::Error::other)?;
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect_lazy(&config.database_url)
        .map_err(io::Error::other)?;

    let bind_address = config.bind_address();
    let allowed_origin = config.cors_allowed_origin.clone();
    let state = web::Data::new(AppState::new(pool, config));

    info!(%bind_address, "starting clottis teams backend");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&allowed_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
            .supports_credentials();

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(Logger::default())
            .configure(routes::configure)
    })
    .bind(bind_address)?
    .run()
    .await
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,actix_web=info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
