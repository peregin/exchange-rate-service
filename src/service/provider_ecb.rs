use crate::route::model::ExchangeRate;
use crate::service::provider::RateProvider;
use chrono::{DateTime, Utc};
use log::info;
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Date;

pub struct EcbRateProvider;

// internal response
#[derive(Serialize, Deserialize, Debug, Clone)]
struct EcbRateHistory {
    pub base: String,
    pub rates: HashMap<String, HashMap<String, f32>>, // date -> rates
}

impl EcbRateProvider {
    // European Central Bank (ECB) rate provider via Frankfurter API
    const HOST: &'static str = "https://api.frankfurter.app";

    pub fn new() -> Self {
        EcbRateProvider
    }

    fn retrieve(&self, path: &str) -> Response {
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
    fn provider_name(&self) -> &'static str {
        "European Central Bank"
    }

    fn latest(&self, base: &str) -> ExchangeRate {
        let reply = self.retrieve(&format!("latest?from={}", base));
        let reply = reply.json::<ExchangeRate>().unwrap();
        info!("base={:#?}, {:#?} rates", base, reply.rates.keys().len());
        reply
    }

    fn symbols(&self) -> HashMap<String, String> {
        let reply = self.retrieve(&String::from("currencies"));
        reply.json::<HashMap<String, String>>().unwrap()
    }

    // wip
    #[allow(unused)]
    fn historical(
        &self,
        base: &str,
        from: &DateTime<Utc>,
        to: &DateTime<Utc>,
    ) -> HashMap<Date, ExchangeRate> {
        let iso_from = from.format("%Y-%m-%d").to_string();
        let iso_to = to.format("%Y-%m-%d").to_string();
        let reply = self.retrieve(&format!("{}..{}?from={}", iso_from, iso_to, base));
        // reply.json::<EcbRateHistory>().unwrap().rates.into_iter().map(|(date, rates)| {
        //     (DateTime::parse_from_str(&date, "%Y-%m-%d").unwrap(), ExchangeRate { base: base.to_owned(), rates })
        // }).map(|(date, rates)| {
        //     (date.date(), rates)
        // }).collect()
        unimplemented!()
    }
}
