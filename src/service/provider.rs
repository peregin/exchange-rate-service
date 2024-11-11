use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::LazyLock;
use log::info;
use time::Date;

use crate::route::model::ExchangeRate;
use crate::service::provider_ecb::EcbRateProvider;
use crate::service::provider_float::FloatRateProvider;

// generic contract what needs to be implemented by any rate provider
pub trait RateProvider: Sync + Send {
    fn provider_name(&self) -> &'static str;

    fn latest(&self, base: &str) -> ExchangeRate;

    // iso3 -> description
    fn symbols(&self) -> HashMap<String, String>;

    fn historical(
        &self,
        base: &str,
        from: &DateTime<Utc>,
        to: &DateTime<Utc>,
    ) -> HashMap<Date, ExchangeRate>;
}

type Providers = Vec<Box<dyn RateProvider>>;

fn get_providers() -> &'static Providers {
    static PROVIDERS: LazyLock<Providers, fn() -> Providers> = LazyLock::new(|| {
        let providers: Providers = vec![
            Box::new(EcbRateProvider::new()),
            Box::new(FloatRateProvider::new()),
        ];
        info!("providers: {:?}", providers.iter().map(|p| p.provider_name()).collect::<Vec<&str>>());
        providers
    });
    &PROVIDERS
}

#[cached(time = 3600)]
pub fn rates_of(base: String) -> ExchangeRate {
    let ecb = EcbRateProvider::new().latest(&base);
    let float = FloatRateProvider::new().latest(&base);
    // ECB rates override float rates
    float.chain(ecb)
}

// map of ISO3 code -> description
#[cached(time = 3600)]
pub fn symbols() -> HashMap<String, String> {
    get_providers()
        .iter()
        .flat_map(|p| p.symbols().into_iter())
        .collect()
}

#[cached(time = 3600)]
pub fn historical_rates_of(
    base: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> HashMap<Date, ExchangeRate> {
    EcbRateProvider::new().historical(&base, &from, &to)
}
