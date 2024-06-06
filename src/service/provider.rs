use awc::Client;
use cached::proc_macro::cached;
use log::info;
use std::collections::HashMap;

use crate::route::model::ExchangeRate;

trait RateProvider {

    async fn rates_of(&self, base: String) -> ExchangeRate;

    async fn symbols(&self) -> HashMap<String, String>;
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

    async fn rates_of(&self, base: String) -> ExchangeRate {
        let client = Client::default();
        let mut reply = client
            .get(format!("{}/latest?from={}", ECBRateProvider::HOST, base))
            .insert_header(("User-Agent", "actix-web"))
            .insert_header(("Content-Type", "application/json"))
            .send()
            .await
            .unwrap();
        let reply = reply.json::<ExchangeRate>().await.unwrap();
        info!("base={:#?}, {:#?} rates", base, reply.rates.keys().len());
        reply
    }

    async fn symbols(&self) -> HashMap<String, String> {
        let client = Client::default();
        let mut reply = client
            .get(format!("{}/currencies", ECBRateProvider::HOST))
            .insert_header(("User-Agent", "actix-web"))
            .insert_header(("Content-Type", "application/json"))
            .send()
            .await
            .unwrap();
        reply.json::<HashMap<String, String>>().await.unwrap()
    }
}

#[cached(time = 3600)]
pub async fn rates_of(base: String) -> ExchangeRate {
    ECBRateProvider::new().rates_of(base).await
}

// map of ISO3 code -> description
#[cached(time = 3600)]
pub async fn symbols() -> HashMap<String, String> {
    ECBRateProvider::new().symbols().await
}


