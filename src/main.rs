mod route;
mod service;

use actix_web::{App, HttpServer};
use actix_cors::Cors;
use log::info;
use regex::Regex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port = option_env!("SERVICE_PORT").unwrap_or("9012");
    info!("starting exchange service on port {port} ...");

    HttpServer::new(|| {
        let allowed_origins = Regex::new(r".*(localhost|peregin\.com|velocorner\.com)").unwrap();
        let cors = Cors::default()
            .allowed_origin_fn(move |origin_header, _request_head| {
                let origin = origin_header.to_str().unwrap();
                info!("origin: {origin}");
                allowed_origins.is_match(origin)
            })
            .allowed_methods(vec!["GET"])
            .allowed_headers(vec![
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE
            ])
            .max_age(3600);
        App::new().wrap(cors).configure(route::route::init_routes)
    })
        .bind(format!("0.0.0.0:{port}"))?
        .run()
        .await
}
