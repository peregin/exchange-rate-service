use actix_web::{get, HttpRequest, Responder, web};
use build_time::build_time_local;
use chrono::{DateTime, Utc};
use std::env;

#[get("/favicon.ico")]
pub async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("static/favicon.ico")?)
}

#[get("/")]
pub async fn welcome(_: HttpRequest) -> impl Responder {
    let now: DateTime<Utc> = Utc::now();
    // Returns the local build timestamp in the specified format.
    let local_build_time = build_time_local!("%Y-%m-%dT%H:%M:%S%.f%:z");
    format!(
        r#"
        <head>
            <title>Exchange Rates</title>
            <link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png">
            <link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png">
            <link rel="icon" type="image/png" sizes="16x16" href="/favicon-16x16.png">
            <link rel="icon" href="/favicon.ico">
            <link rel="manifest" href="/site.webmanifest">
        </head>
        <body>
            <h1>Welcome to Exchange Rate Service 🚀🪙</h1>
            Current time is <i>{}</i><br/>
            Build time is <i>{}</i><br/>
            OS type is <i>{} {}</i><br/>
            Open API <a href="/docs/">/docs</a><br/>
        </body>
    "#,
        now,
        local_build_time,
        env::consts::OS,
        env::consts::ARCH
    )
        .customize()
        .insert_header(("content-type", "text/html; charset=utf-8"))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(welcome);
    config.service(favicon);
}