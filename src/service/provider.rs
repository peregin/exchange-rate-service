use awc::Client;
use cached::proc_macro::cached;
use log::info;
use std::collections::HashMap;

use crate::route::model::ExchangeRate;

// pub trait RateProvider {
//     async fn rates_of(&self, base: String) -> ExchangeRate;
//
//     async fn symbols(&self) -> HashMap<String, String>;
// }
//
// struct FloatRateProvider;
//
// impl FloatRateProvider {
//     fn new() -> Self {
//         FloatRateProvider
//     }
// }

// European Central Bank (ECB) rate provider via Frankfurter API
const HOST: &str = "https://api.frankfurter.app";

#[cached(time = 3600)]
pub async fn rates_of(base: String) -> ExchangeRate {
    let client = Client::default();
    let mut reply = client
        .get(format!("{}/latest?from={}", HOST, base))
        .insert_header(("User-Agent", "actix-web"))
        .insert_header(("Content-Type", "application/json"))
        .send()
        .await
        .unwrap();
    let reply = reply.json::<ExchangeRate>().await.unwrap();
    info!("base={:#?}, {:#?} rates", base, reply.rates.keys().len());
    reply
}

// map of ISO3 code -> description
#[cached(time = 3600)]
pub async fn symbols() -> HashMap<String, String> {
    let client = Client::default();
    let mut reply = client
        .get(format!("{}/currencies", HOST))
        .insert_header(("User-Agent", "actix-web"))
        .insert_header(("Content-Type", "application/json"))
        .send()
        .await
        .unwrap();
    reply.json::<HashMap<String, String>>().await.unwrap()
}


