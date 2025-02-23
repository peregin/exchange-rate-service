use crate::service::provider::count_providers;
use actix_web::{get, web, HttpRequest, Responder};
use actix_files::NamedFile;
use build_timestamp::build_time;
use humansize::{format_size, DECIMAL};
use std::env;
use sysinfo::System;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};

#[get("/favicon.ico")]
pub async fn favicon(_: HttpRequest) -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

#[get("/")]
pub async fn welcome(_: HttpRequest) -> impl Responder {
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc2822)
        .unwrap();
    // generates a timestamp in const BUILD_TIME as string slice
    build_time!("%Y-%m-%d %H:%M:%S");
    // println!("BUILD_TIME: {}", BUILD_TIME);
    let built = PrimitiveDateTime::parse(
        "2024-11-11 20:12:28",
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    )
    .unwrap()
    .assume_utc()
    .format(&time::format_description::well_known::Rfc2822)
    .unwrap();
    // calculate uptime
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // memory info
    let mut sys = System::new_all();
    sys.refresh_all();
    format!(
        r#"
        <head>
            <title>Exchange Rates</title>
            <link rel="apple-touch-icon" sizes="180x180" href="/static/apple-touch-icon.png">
            <link rel="icon" type="image/png" sizes="32x32" href="/static/favicon-32x32.png">
            <link rel="icπon" type="image/png" sizes="16x16" href="/static/favicon-16x16.png">
            <link rel="icon" href="/static/favicon.ico">
            <link rel="manifest" href="/static/site.webmanifest">
        </head>
        <body>
            <h1>Welcome to Exchange Rate Service 🚀🪙</h1>
            Current time: <i>{}</i><br/>
            Build time: <i>{}</i><br/>
            Uptime: <i>{}</i><br/>
            OS type: <i>{} {}</i><br/>
            Used/total memory: <i>{} / {}</i><br/>
            Providers: <i>{}</i><br/>
            Open API <a href="/docs/">/docs</a><br/>
        </body>
    "#,
        now,
        built,
        format!(
            "{:02}:{:02}:{:02}",
            uptime / 3600,
            (uptime % 3600) / 60,
            uptime % 60
        ),
        env::consts::OS,
        env::consts::ARCH,
        format_size(sys.used_memory(), DECIMAL),
        format_size(sys.total_memory(), DECIMAL),
        count_providers(),
    )
    .customize()
    .insert_header(("content-type", "text/html; charset=utf-8"))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(welcome);
    config.service(favicon);
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::dev::ServiceResponse;
    use actix_web::{http::header, test, App};

    #[actix_web::test]
    async fn test_welcome_endpoint() {
        // Create test app
        let app = test::init_service(App::new().configure(init_routes)).await;

        // Create test request
        let req = test::TestRequest::get().uri("/").to_request();

        // Execute request
        let resp: ServiceResponse = test::call_service(&app, req).await;

        // Assert response status
        assert_eq!(resp.status(), 200);

        // Check content type header
        let content_type = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(content_type, "text/html; charset=utf-8");

        // Get response body
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Assert body contains expected elements
        assert!(body_str.contains("<title>Exchange Rates</title>"));
        assert!(body_str.contains("<h1>Welcome to Exchange Rate Service 🚀🪙</h1>"));
        assert!(body_str.contains("Current time:"));
        assert!(body_str.contains("Build time:"));
        assert!(body_str.contains("Uptime:"));
        assert!(body_str.contains("OS type:"));
        assert!(body_str.contains("Used/total memory:"));
        assert!(body_str.contains("Providers:"));
        assert!(body_str.contains(r#"<a href="/docs/">/docs</a>"#));
    }

    #[actix_web::test]
    async fn test_favicon_endpoint() {
        let app = test::init_service(App::new().configure(init_routes)).await;

        let req = test::TestRequest::get().uri("/favicon.ico").to_request();

        let resp: ServiceResponse = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);

        let content_type = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(content_type, "image/x-icon");
    }

    #[test]
    async fn test_uptime_format() {
        // Test the uptime formatting logic
        let test_cases = vec![
            (3600, "01:00:00"),  // 1 hour
            (3665, "01:01:05"),  // 1 hour, 1 minute, 5 seconds
            (7200, "02:00:00"),  // 2 hours
            (86399, "23:59:59"), // 23 hours, 59 minutes, 59 seconds
            (0, "00:00:00"),     // 0 seconds
        ];

        for (seconds, expected) in test_cases {
            let formatted = format!(
                "{:02}:{:02}:{:02}",
                seconds / 3600,
                (seconds % 3600) / 60,
                seconds % 60
            );
            assert_eq!(formatted, expected);
        }
    }
}
