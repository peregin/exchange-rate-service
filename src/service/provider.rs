use cached::proc_macro::cached;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use time::Date;

use crate::route::model::ExchangeRate;
use crate::service::provider_ecb::EcbRateProvider;
use crate::service::provider_float::FloatRateProvider;

// generic contract what needs to be implemented by any rate provider
pub trait RateProvider {
    fn latest(&self, base: &String) -> ExchangeRate;

    // iso3 -> description
    fn symbols(&self) -> HashMap<String, String>;

    fn historical(&self, base: &String, from: &DateTime<Utc>, to: &DateTime<Utc>) -> HashMap<Date, ExchangeRate>;
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
    let ecb = EcbRateProvider::new().symbols();
    let float = FloatRateProvider::new().symbols();
    // merge 2 hashmaps with the supported symbols together
    ecb.into_iter().chain(float.into_iter()).collect()
}

#[cached(time = 3600)]
pub fn historical_rates_of(base: String, from: DateTime<Utc>, to: DateTime<Utc>) -> HashMap<Date, ExchangeRate> {
    EcbRateProvider::new().historical(&base, &from, &to)
}


