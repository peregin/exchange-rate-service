use reqwest::blocking::Client;
use cached::proc_macro::cached;
use log::info;
use std::collections::HashMap;

use crate::route::model::ExchangeRate;

pub trait RateProvider {

    fn rates_of(&self, base: String) -> ExchangeRate;

    fn symbols(&self) -> HashMap<String, String>;
}

struct FloatRateProvider;

impl FloatRateProvider {
    fn new() -> Self {
        FloatRateProvider
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

    fn rates_of(&self, base: String) -> ExchangeRate {
        let client = Client::new();
        let mut reply = client
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
        let mut reply = client
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
    ECBRateProvider::new().rates_of(base)
}

// map of ISO3 code -> description
#[cached(time = 3600)]
pub fn symbols() -> HashMap<String, String> {
    ECBRateProvider::new().symbols()
}


