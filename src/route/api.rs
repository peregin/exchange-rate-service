use crate::route::model::ExchangeRate;
use crate::service::provider::{historical_rates_of, rates_of, symbols};
use actix_web::rt::task::spawn_blocking;
use actix_web::{get, web, HttpResponse, Responder};
use time::{Date, Duration, OffsetDateTime};
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
    let (now, last_month) = history_range(30);
    let series = spawn_blocking(move || {
        // map keys can be String only!!! convert Date to String
        historical_rates_of(base, last_month, now)
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
        ("counter" = String, Path, example = "EUR"),
    ),
    responses(
        (
        status = 200,
        body = HashMap < time::Date, f32 >,
        description = "Time series of the exchange rate, back to given days or today",
        example = json ! ({"2024-11-10": 1.1204, "2024-11-11": 1.0411, "2024-11-12": 1.0918})
        )
    )
)]
#[get("/api/rates/historical/{base}/{counter}")]
async fn historical_rate(params: web::Path<(String, String)>) -> HttpResponse {
    let (base, counter) = params.into_inner();
    let base = base.to_uppercase();
    let counter = counter.to_uppercase();
    let (now, last_month) = history_range(30);
    let series = spawn_blocking(move || {
        // map keys can be String only!!! convert Date to String
        historical_rates_of(base, last_month, now)
            .iter()
            .flat_map(|(k, ex)| ex.rates.get(&counter).map(|r| (k, r)))
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect::<std::collections::BTreeMap<_, _>>()
    })
    .await
    .unwrap();
    HttpResponse::Ok().json(series)
}

fn history_range(days: i64) -> (Date, Date) {
    let now = OffsetDateTime::now_utc().date();
    let last_month = now - Duration::days(days);
    (now, last_month)
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
        ),
        (
        status = 404,
        description = "No exchange rate found"
        )
    )
)]
#[get("/api/rates/{base}/{counter}")]
async fn rate(params: web::Path<(String, String)>) -> HttpResponse {
    let (base, counter) = params.into_inner();
    let base = base.to_uppercase();
    let counter = counter.to_uppercase();
    let exchanges = spawn_blocking(move || rates_of(base))
        .await
        .unwrap();
    match exchanges.rates.get(&counter) {
        Some(fx) => HttpResponse::Ok().json(fx),
        None => HttpResponse::NotFound().finish(),
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Exchange Rates API",
        description = "Rates API description",
        version = "1.0.0",
        contact(name = "peregin.com", email = "hello@peregin.com"),
        license(name = "MIT", url = "https://opensource.org/license/mit")
    ),
    paths(
        currencies,
        rates,
        rate,
        historical_rates,
        historical_rate,
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
    config.service(historical_rate);
    config.service(rates);
    config.service(rate);
    config.service(SwaggerUi::new("/docs/{_:.*}").url("/opanapi.json", ApiDoc::openapi()));
}
