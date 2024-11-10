use crate::route::model::ExchangeRate;
use crate::service::provider::RateProvider;
use chrono::{DateTime, Utc};
use log::info;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Date;

pub struct FloatRateProvider;

// internal response
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FloatRateEntry {
    pub code: String,
    pub name: String,
    pub rate: f32,
}

impl FloatRateProvider {
    const HOST: &'static str = "https://www.floatrates.com";

    pub fn new() -> Self {
        FloatRateProvider
    }

    fn retrieve(&self, base: &str) -> Vec<FloatRateEntry> {
        let client = Client::new();
        let reply = client
            .get(format!(
                "{}/daily/{}.json",
                FloatRateProvider::HOST,
                base.to_lowercase()
            ))
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
    fn provider_name(&self) -> &'static str {
        "floatrates.com"
    }

    // latest exchange rate

    fn latest(&self, base: &str) -> ExchangeRate {
        let reply = self.retrieve(base);
        ExchangeRate {
            base: base.to_owned(),
            rates: reply.into_iter().map(|e| (e.code, e.rate)).collect(),
        }
    }

    fn symbols(&self) -> HashMap<String, String> {
        self.retrieve(&String::from("CHF"))
            .into_iter()
            .map(|e| (e.code, e.name))
            .collect()
    }

    fn historical(
        &self,
        _base: &String,
        _from: &DateTime<Utc>,
        _to: &DateTime<Utc>,
    ) -> HashMap<Date, ExchangeRate> {
        unimplemented!()
    }
}
