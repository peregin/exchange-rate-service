use crate::model::ExchangeRate;
use crate::service::client::{rates_of, symbols};
use crate::route::{welcome, favicon};

use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    config.service(welcome);
    config.service(favicon);
    config.service(currencies);
    config.service(rates);
    config.service(rate);
    config.service(SwaggerUi::new("/docs/{_:.*}").url("/opanapi.json", ApiDoc::openapi()));
    config.service(actix_files::Files::new("/static", "static").show_files_listing());
}
