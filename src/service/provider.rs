use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use log::info;
use std::collections::HashMap;
use std::sync::LazyLock;
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
        // sequence is important, the latter will override the same currencies
        // ECB rates override float rates
        let providers: Providers = vec![
            Box::new(EcbRateProvider::new()),
            Box::new(FloatRateProvider::new()),
        ];
        info!(
            "providers: {:?}",
            providers
                .iter()
                .map(|p| p.provider_name())
                .collect::<Vec<&str>>()
        );
        providers
    });
    &PROVIDERS
}

#[cached(time = 3600)]
pub fn rates_of(base: String) -> ExchangeRate {
    let rates = get_providers()
        .iter()
        .map(|p| p.latest(&base));
    // merge with priority (ECB rates overrides floating rates)
    rates.fold(ExchangeRate::empty(&base), |acc, current| current.chain(acc))
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
