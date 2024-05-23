use crate::model::ExchangeRate;
use crate::service::{rates_of, symbols};
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use std::env;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// root path, simple welcome message
async fn welcome(_: HttpRequest) -> impl Responder {
    let now: DateTime<Utc> = Utc::now();
    format!(
        r#"
    Welcome to <b>Exchange Rate Service</b> 🚀🪙<br/>
    Current time is <i>{}</i><br/>
    OS type is <i>{} {}</i><br/>
    Open API <a href="/docs/">/docs</a><br/>
    "#,
        now,
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
}
