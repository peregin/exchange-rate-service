mod route;
mod service;

use actix_web::{App, HttpServer};
use actix_cors::Cors;
use log::{debug, info};
use regex::Regex;
use actix_web::http::header;

const ALLOWED_ORIGINS: &str = r".*(localhost|peregin\.com|velocorner\.com)";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port = option_env!("SERVICE_PORT").unwrap_or("9012");
    info!("starting exchange service on port {port} ...");

    HttpServer::new(|| {
        let origins_regex = Regex::new(ALLOWED_ORIGINS).unwrap();
        let cors = Cors::default()
            .allowed_origin_fn(move |origin_header, _request_head| {
                let origin = origin_header.to_str().unwrap();
                info!("origin: {origin}");
                is_allowed_origin(origin, &origins_regex)
            })
            .allow_any_header()
            .allow_any_method()
            .expose_any_header()
            .max_age(3600); // preflight cache TTL
        App::new().wrap(cors).configure(route::route::init_routes)
    })
        .bind(format!("0.0.0.0:{port}"))?
        .run()
        .await
}

fn is_allowed_origin(origin: &str, origins_regex: &Regex) -> bool {
    origins_regex.is_match(origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_origin() {
        let origins_regex = Regex::new(ALLOWED_ORIGINS).unwrap();

        // Test allowed origins
        assert!(is_allowed_origin("https://www.peregin.com", &origins_regex));
        assert!(is_allowed_origin("https://rates.velocorner.com", &origins_regex));
        assert!(is_allowed_origin("http://localhost:8000", &origins_regex));

        // Test disallowed origins
        assert!(!is_allowed_origin("https://www.example.org", &origins_regex));
        assert!(!is_allowed_origin("http://127.0.0.1", &origins_regex));
    }
}
