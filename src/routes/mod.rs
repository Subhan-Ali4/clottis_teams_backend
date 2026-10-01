mod health_routes;

use actix_web::web;

pub fn configure(config: &mut web::ServiceConfig) {
    config.service(web::scope("/api/v1").configure(health_routes::configure));
}
