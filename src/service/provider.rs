use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use time::Date;

use crate::route::model::ExchangeRate;
use crate::service::provider_ecb::EcbRateProvider;
use crate::service::provider_float::FloatRateProvider;

// generic contract what needs to be implemented by any rate provider
pub trait RateProvider: Sync + Send /*+ Hash + Eq*/ {
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

#[cached(time = 3600)]
pub fn rates_of(base: String) -> ExchangeRate {
    let ecb = EcbRateProvider::new().latest(&base);
    let float = FloatRateProvider::new().latest(&base);
    // ECB rates override float rates
    float.chain(ecb)
}

// map of ISO3 code -> description
// TODO: to make it cacheable
// use caching on individual providers - level
// #[cached(time = 3600)]
pub fn symbols(providers: &[Box<dyn RateProvider>]) -> HashMap<String, String> {
    providers
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
