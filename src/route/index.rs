use crate::service::provider::count_providers;
use actix_web::{get, web, HttpRequest, Responder};
use build_timestamp::build_time;
use humansize::{format_size, DECIMAL};
use std::env;
use sysinfo::System;
use time::macros::format_description;
use time::{OffsetDateTime, PrimitiveDateTime};

#[get("/favicon.ico")]
pub async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("/static/favicon.ico")?)
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
        format!("{:02}:{:02}:{:02}", uptime / 3600, (uptime % 3600) / 60, uptime % 60),
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
