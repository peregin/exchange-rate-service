use crate::route::model::ExchangeRate;
use crate::service::provider::{historical_rates_of, rates_of, symbols};
use actix_web::rt::task::spawn_blocking;
use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use std::arch::x86_64::_mm_add_pd;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[utoipa::path(
    get,
    tag = "rates",
    responses(
        (status = 200, description = "List supported currencies",
        body = HashMap < String, String >,
        example = json ! ({"CHF": "Swiss Franc", "USD": "U.S. Dollar", "EUR": "Euro", "KES": "Kenyan shilling"}))
    )
)]
#[get("/api/rates/currencies")]
async fn currencies() -> impl Responder {
    spawn_blocking(move || {
        let pairs = symbols();
        let sorted = pairs
            .iter()
            .map(|(k, v)| (k.to_uppercase(), v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        web::Json(sorted)
    })
    .await
    .unwrap()
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
        body = HashMap < time::Date, ExchangeRate >,
        description = "Time series of the exchange rates, back to given days or today",
        )
    )
)]
#[get("/api/rates/historical/{base}")]
async fn historical_rates(params: web::Path<String>) -> HttpResponse {
    let base = params.into_inner().to_uppercase();
    let now: DateTime<Utc> = Utc::now();
    let last_month: DateTime<Utc> = now - chrono::Duration::days(30);
    let series = spawn_blocking(move || {
        // map keys can be String only!!! convert Date to String
        historical_rates_of(base.to_uppercase(), last_month, now)
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    })
    .await
    .unwrap();
    HttpResponse::Ok().json(series)
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
        body = ExchangeRate,
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
    })
    .await
    .unwrap()
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
        description = "Actual exchange rate for the given base and counter currencies",
        body = f32,
        example = json ! (1.0305)
        )
    )
)]
#[get("/api/rates/{base}/{counter}")]
async fn rate(params: web::Path<(String, String)>) -> HttpResponse {
    let (base, counter) = params.into_inner();
    let exchanges = spawn_blocking(move || rates_of(base.to_uppercase()))
        .await
        .unwrap();
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
        historical_rates,
    ),
    components(schemas(ExchangeRate)),
    tags(
        (name = "rates", description = "Exchange rates")
    ),
)]
struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(currencies);
    config.service(historical_rates); // must be defined earlier, otherwise path is considered as parameter (historical={base})
    config.service(rates);
    config.service(rate);
    config.service(SwaggerUi::new("/docs/{_:.*}").url("/opanapi.json", ApiDoc::openapi()));
}
