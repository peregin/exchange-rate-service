mod model;
mod route;
mod service;

use actix_web::{App, HttpServer};
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port = option_env!("SERVICE_PORT").unwrap_or("9012");
    info!("starting exchange service on port {port} ...");

    HttpServer::new(|| App::new().configure(route::route::init_routes))
        .bind(format!("0.0.0.0:{port}"))?
        .run()
        .await
}
