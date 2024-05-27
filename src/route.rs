use crate::model::ExchangeRate;
use crate::service::{rates_of, symbols};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use build_time::build_time_local;
use chrono::{DateTime, Utc};
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[get("/favicon.ico")]
async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
    Ok(actix_files::NamedFile::open("static/favicon.ico")?)
}

// root path, simple welcome message
async fn welcome(_: HttpRequest) -> impl Responder {
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

#[utoipa::path(
    responses(
        (status = 200, description = "List supported currencies", body = [String])
    )
)]
#[get("/rates/currencies")]
async fn currencies() -> impl Responder {
    web::Json(symbols().await.keys().cloned().collect::<Vec<_>>())
}

#[get("/rates/{base}")]
async fn rates(info: web::Path<String>) -> impl Responder {
    let base = info.into_inner().to_uppercase();
    let exchanges = rates_of(String::from(base)).await;
    web::Json(exchanges)
}

#[get("/rates/{base}/{counter}")]
async fn rate(params: web::Path<(String, String)>) -> HttpResponse {
    let (base, counter) = params.into_inner();
    let exchanges = rates_of(base.to_uppercase()).await;
    match exchanges.rates.get(&counter.to_uppercase()) {
        Some(fx) => HttpResponse::Ok().json(fx),
        None => HttpResponse::NotFound().finish(),
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Exchange Rates API",
        description = "Rates API description"
    ),
    paths(
        currencies,
    ),
    components(schemas(
        ExchangeRate
    )),
    tags(
        (name = "rates", description = "Exchange rates")
    ),
)]
struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.route("/", web::get().to(welcome));
    config.service(currencies);
    config.service(rates);
    config.service(rate);
    config.service(SwaggerUi::new("/docs/{_:.*}").url("/opanapi.json", ApiDoc::openapi()));
    config.service(actix_files::Files::new("/static", "static").show_files_listing());
    config.service(favicon);
}
