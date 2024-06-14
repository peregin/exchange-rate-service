use reqwest::blocking::Client;
use cached::proc_macro::cached;
use log::info;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::route::model::ExchangeRate;

pub trait RateProvider {
    fn rates_of(&self, base: &String) -> ExchangeRate;

    fn symbols(&self) -> HashMap<String, String>;
}

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

    fn rates_of(&self, base: &String) -> ExchangeRate {
        let reply = self.retrieve(base);
        ExchangeRate {
            base: base.to_owned(),
            rates: reply.into_iter().map(|e| (e.code, e.rate)).collect(),
        }
    }

    fn symbols(&self) -> HashMap<String, String> {
        self.retrieve(&String::from("CHF")).into_iter().map(|e| (e.code, e.name)).collect()
    }
}

pub struct ECBRateProvider;

impl ECBRateProvider {
    // European Central Bank (ECB) rate provider via Frankfurter API
    const HOST: &'static str = "https://api.frankfurter.app";

    pub fn new() -> Self {
        ECBRateProvider
    }
}

impl RateProvider for ECBRateProvider {
    fn rates_of(&self, base: &String) -> ExchangeRate {
        let client = Client::new();
        let reply = client
            .get(format!("{}/latest?from={}", ECBRateProvider::HOST, base))
            .header("User-Agent", "actix-web")
            .header("Content-Type", "application/json")
            .send()
            .unwrap();
        let reply = reply.json::<ExchangeRate>().unwrap();
        info!("base={:#?}, {:#?} rates", base, reply.rates.keys().len());
        reply
    }

    fn symbols(&self) -> HashMap<String, String> {
        let client = Client::new();
        let reply = client
            .get(format!("{}/currencies", ECBRateProvider::HOST))
            .header("User-Agent", "actix-web")
            .header("Content-Type", "application/json")
            .send()
            .unwrap();
        reply.json::<HashMap<String, String>>().unwrap()
    }
}

#[cached(time = 3600)]
pub fn rates_of(base: String) -> ExchangeRate {
    let ecb = ECBRateProvider::new().rates_of(&base);
    let float = FloatRateProvider::new().rates_of(&base);
    // ECB rates override float rates
    float.chain(ecb)
}

// map of ISO3 code -> description
#[cached(time = 3600)]
pub fn symbols() -> HashMap<String, String> {
    let ecb = ECBRateProvider::new().symbols();
    let float = FloatRateProvider::new().symbols();
    // merge 2 hashmaps with the supported symbols together
    ecb.into_iter().chain(float.into_iter()).collect()
}


