use actix_web::{get, HttpResponse, Responder, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::service::provider::{symbols, rates_of};
use crate::model::ExchangeRate;

#[utoipa::path(
    get,
    tag = "rates",
    responses(
        (status = 200, description = "List supported currencies", body = [String], example = json!(["CHF", "USD", "EUR", "KES"]))
    )
)]
#[get("/api/rates/currencies")]
async fn currencies() -> impl Responder {
    let mut syms = symbols().await.keys().cloned().collect::<Vec<_>>();
    syms.sort();
    web::Json(syms)
}

#[utoipa::path(
    get,
    tag = "rates",
    params(
        ("base" = String, Path, example = "CHF"),
    ),
    responses(
        (
            status = 200,
            description = "List actual exchange rates with the given base currency",
            body = [ExchangeRate],
            example = json!({"base": "CHF", "rates": {"USD": 1.1204, "EUR": 1.0305, "JPY": 174.9}})
        )
    )
)]
#[get("/api/rates/{base}")]
async fn rates(info: web::Path<String>) -> impl Responder {
    let base = info.into_inner().to_uppercase();
    let exchanges = rates_of(String::from(base)).await;
    web::Json(exchanges)
}

#[utoipa::path(
    get,
    tag = "rates",
    params(
        ("base" = String, Path, example = "CHF"),
        ("counter" = String, Path, example = "EUR"),
    ),
    responses(
        (
            status = 200,
            description = "List actual exchange rate for the given base and counter currencies",
            body = [f32],
            example = json!(1.0305)
        )
    )
)]
#[get("/api/rates/{base}/{counter}")]
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
        rates,
        rate,
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
    config.service(currencies);
    config.service(rates);
    config.service(rate);
    config.service(SwaggerUi::new("/docs/{_:.*}").url("/opanapi.json", ApiDoc::openapi()));
}