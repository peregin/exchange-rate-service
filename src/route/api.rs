use actix_web::{get, HttpResponse, Responder, web};
use actix_web::rt::task::spawn_blocking;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::service::provider::{symbols, rates_of, latest_rates_of};
use crate::route::model::ExchangeRate;

#[utoipa::path(
    get,
    tag = "rates",
    responses(
        (status = 200, description = "List supported currencies", body = HashMap < String, String >, example = json ! ({"CHF": "Swiss Franc", "USD": "U.S. Dollar", "EUR": "Euro", "KES": "Kenyan shilling"}))
    )
)]
#[get("/api/rates/currencies")]
async fn currencies() -> impl Responder {
    spawn_blocking(move || {
        let pairs = symbols();
        let sorted = pairs.iter().map(|(k, v)| (k.to_uppercase(), v.clone())).collect::<std::collections::BTreeMap<_, _>>();
        web::Json(sorted)
    }).await.unwrap()
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
        example = json ! ({"base": "CHF", "rates": {"USD": 1.1204, "EUR": 1.0305, "JPY": 174.9}})
        )
    )
)]
#[get("/api/rates/{base}")]
async fn rates(info: web::Path<String>) -> impl Responder {
    let base: String = info.into_inner().to_uppercase();
    spawn_blocking(move || {
        let exchanges = rates_of(base);
        web::Json(exchanges)
    }).await.unwrap()
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
        example = json ! (1.0305)
        )
    )
)]
#[get("/api/rates/{base}/{counter}")]
async fn rate(params: web::Path<(String, String)>) -> HttpResponse {
    let (base, counter) = params.into_inner();
    let exchanges = spawn_blocking(move || {
        rates_of(base.to_uppercase())
    }).await.unwrap();
    match exchanges.rates.get(&counter.to_uppercase()) {
        Some(fx) => HttpResponse::Ok().json(fx),
        None => HttpResponse::NotFound().finish(),
    }
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
        description = "Time series of the exchange rates, back to the given days or today",
        )
    )
)]
#[get("/api/rates/latest/{base}")]
async fn last_rate(params: web::Path<String>) -> HttpResponse {
    let base = params.into_inner().to_uppercase();
    let series = spawn_blocking(move || {
        latest_rates_of(base.to_uppercase(), 1)
    }).await.unwrap();
    HttpResponse::Ok().json(series)
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
        last_rate,
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
    // would it make sense to have the providers as application data?
    //let provider: Box<dyn crate::service::provider::RateProvider> = Box::new(ECBRateProvider::new());
    //config.app_data(web::Data::new(provider));

    config.service(currencies);
    config.service(rates);
    config.service(rate);
    config.service(last_rate);
    config.service(SwaggerUi::new("/docs/{_:.*}").url("/opanapi.json", ApiDoc::openapi()));
}