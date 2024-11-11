use crate::route::model::ExchangeRate;
use crate::service::provider::RateProvider;
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
        _base: &str,
        _from: &Date,
        _to: &Date,
    ) -> HashMap<Date, ExchangeRate> {
        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use time::Month::November;
    use super::*;

    #[test]
    fn test_historical_empty_response() {
        let provider = FloatRateProvider::new(); // Replace with your actual provider struct
        let base = "USD";
        let from = Date::from_calendar_date(2023, time::Month::January, 1).unwrap();
        let to= Date::from_calendar_date(2024, November, 11).unwrap();

        let result = provider.historical(base, &from, &to);

        assert!(result.is_empty());
    }

    #[test]
    fn test_historical_date_range() {
        let provider = FloatRateProvider::new();
        let base = "EUR";
        let from = Date::from_calendar_date(2024, November, 11).unwrap();
        let to = from + time::Duration::days(10);

        let result = provider.historical(base, &from, &to);

        assert!(result.is_empty());
    }
}
