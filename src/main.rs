use std::io::Write;
mod route;
mod service;

use actix_web::{http, App, HttpServer};
use actix_cors::Cors;
use log::{debug, info};
use regex::Regex;

use std::sync::LazyLock;
use actix_web::dev::ServiceResponse;
use actix_web::http::header::HeaderValue;
use actix_web::http::StatusCode;
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use time::OffsetDateTime;

const NA: &'static str = "n/a";

// TODO: use async all the way down -> then caching sort out differently
// TODO: don't use unwrap
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().format(|buf, record| {
        writeln!(
            buf,
            "[{}] {}: {}",
            OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap_or(NA.to_string()),
            record.level(),
            record.args()
        )
    }).init();
    let port = option_env!("SERVICE_PORT").unwrap_or("9012");
    info!("starting exchange service on port {port} ...");

    HttpServer::new(|| {
        let cors = Cors::permissive()
            .allowed_origin_fn(move |origin_header, _request_head| {
                let origin = origin_header.to_str().unwrap();
                debug!("origin: {origin}");
                is_allowed_origin(origin)
            });
        App::new()
            .wrap(cors)
            .wrap(ErrorHandlers::new().handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500))
            .configure(route::route::init_routes)
    })
        .bind(format!("0.0.0.0:{port}"))?
        .run()
        .await
}

fn render_500<B, E>(mut res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>, E> {
    let response = res.response_mut();

    // Set proper content type for error response
    response.headers_mut().insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    // Set proper status code if not already set
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;

    // If you want to modify the body, you'll need to handle the body type appropriately
    // This depends on your specific needs and the body type B
    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
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
