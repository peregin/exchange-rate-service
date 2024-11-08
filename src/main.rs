mod route;
mod service;

use actix_web::{App, HttpServer};
use actix_cors::Cors;
use log::{debug, info};
use regex::Regex;

use std::sync::LazyLock;

// TODO: use async all the way down -> then caching sort out differently
// TODO: don't use unwrap
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let port = option_env!("SERVICE_PORT").unwrap_or("9012");
    info!("starting exchange service on port {port} ...");

    HttpServer::new(|| {
        let cors = Cors::permissive()
            .allowed_origin_fn(move |origin_header, _request_head| {
                let origin = origin_header.to_str().unwrap();
                debug!("origin: {origin}");
                is_allowed_origin(origin)
            });
        App::new().wrap(cors).configure(route::route::init_routes)
    })
        .bind(format!("0.0.0.0:{port}"))?
        .run()
        .await
}


fn is_allowed_origin(origin: &str) -> bool {
    const ALLOWED_ORIGINS: &str = r".*(localhost|peregin\.com|velocorner\.com)";
    static ORIGINS_REGEX: LazyLock<Regex, fn() -> Regex> = LazyLock::new(|| Regex::new(ALLOWED_ORIGINS).unwrap());
    ORIGINS_REGEX.is_match(origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_allowed_origin() {
        // Test allowed origins
        assert!(is_allowed_origin("https://www.peregin.com"));
        assert!(is_allowed_origin("https://rates.velocorner.com"));
        assert!(is_allowed_origin("http://localhost:8000"));
        assert!(is_allowed_origin("http://localhost:3000"));

        // Test disallowed origins
        assert!(!is_allowed_origin("https://www.example.org"));
        assert!(!is_allowed_origin("http://127.0.0.1"));
    }
}
