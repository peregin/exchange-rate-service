use reqwest::blocking::{Client, Response};
use cached::proc_macro::cached;
use log::info;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::route::model::ExchangeRate;

// generic contract what needs to be implemented by any rate provider
pub trait RateProvider {
    fn latest(&self, base: &String) -> ExchangeRate;

    // iso3 -> description
    fn symbols(&self) -> HashMap<String, String>;

    fn historical(&self, base: &String, from: &DateTime<Utc>, to: &DateTime<Utc>) -> HashMap<Date, ExchangeRate>;
}

// specific implementation backed up by float rate provider
struct FloatRateProvider;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FloatRateEntry {
    pub code: String,
    pub name: String,
    pub rate: f32,
}

impl FloatRateProvider {
    const HOST: &'static str = "https://www.floatrates.com";

    fn new() -> Self {
        FloatRateProvider
    }

    fn retrieve(&self, base: &String) -> Vec<FloatRateEntry> {
        let client = Client::new();
        let reply = client
            .get(format!("{}/daily/{}.json", FloatRateProvider::HOST, base.to_lowercase()))
            .header("User-Agent", "actix-web")
            .header("Content-Type", "application/json")
            .send()
            .unwrap();
        let reply = reply.json::<HashMap<String, FloatRateEntry>>().unwrap();
        info!("base={:#?}, {:#?} rates", base, reply.len());
        reply.values().cloned().collect()
    }
}

impl RateProvider for FloatRateProvider {
    fn latest(&self, base: &String) -> ExchangeRate {
        let reply = self.retrieve(base);
        ExchangeRate {
            base: base.to_owned(),
            rates: reply.into_iter().map(|e| (e.code, e.rate)).collect(),
        }
    }

    fn symbols(&self) -> HashMap<String, String> {
        self.retrieve(&String::from("CHF")).into_iter().map(|e| (e.code, e.name)).collect()
    }

    fn historical(&self, _base: &String, _from: &DateTime<Utc>, _to: &DateTime<Utc>) -> HashMap<Date, ExchangeRate> {
        unimplemented!()
    }
}

// specific implementation backed up by ECB rates
pub struct EcbRateProvider;

impl EcbRateProvider {
    // European Central Bank (ECB) rate provider via Frankfurter API
    const HOST: &'static str = "https://api.frankfurter.app";

    pub fn new() -> Self {
        EcbRateProvider
    }

    fn get(&self, path: &String) -> Response {
        let client = Client::new();
        client
            .get(format!("{}/{}", EcbRateProvider::HOST, path))
            .header("User-Agent", "actix-web")
            .header("Content-Type", "application/json")
            .send()
            .unwrap()
    }
}

impl RateProvider for EcbRateProvider {
    fn latest(&self, base: &String) -> ExchangeRate {
        let reply = self.get(&format!("latest?from={}", base));
        let reply = reply.json::<ExchangeRate>().unwrap();
        info!("base={:#?}, {:#?} rates", base, reply.rates.keys().len());
        reply
    }

    fn symbols(&self) -> HashMap<String, String> {
        let reply = self.get(&String::from("currencies"));
        reply.json::<HashMap<String, String>>().unwrap()
    }

    fn historical(&self, base: &String, from: &DateTime<Utc>, to: &DateTime<Utc>) -> HashMap<Date, ExchangeRate> {
        let iso_from = from.format("%Y-%m-%d").to_string();
        let iso_to = to.format("%Y-%m-%d").to_string();
        let reply = self.get(&format!("{}..{}?from={}", iso_from, iso_to, base));
        //reply.json::
        HashMap::new()
    }
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


