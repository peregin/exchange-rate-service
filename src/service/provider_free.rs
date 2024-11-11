use crate::route::model::ExchangeRate;
use crate::service::provider::RateProvider;
use reqwest::blocking::{Client, Response};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use time::Date;
use time::format_description::well_known::Iso8601;

pub struct FreeRateProvider;

// internal response
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FreeRateEntry {
    date: String,
    // Map to handle dynamic top-level currency keys
    #[serde(flatten)]
    currencies: HashMap<String, HashMap<String, f32>>,
}

impl FreeRateProvider {
    // fast, free, no rate limit via CDN
    const HOST: &'static str = "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api";

    pub fn new() -> Self {
        FreeRateProvider
    }

    fn retrieve(&self, path: &str) -> Response {
        let client = Client::new();
        client
            .get(format!("{}@{}", FreeRateProvider::HOST, path))
            .header("User-Agent", "actix-web")
            .header("Content-Type", "application/json")
            .send()
            .unwrap()
    }

    fn rates_from(
        &self,
        base: &str,
        at: &Date,
    ) -> ExchangeRate {
        let format = Iso8601::DATE;
        let iso_at = at.format(&format).unwrap();
        let key = base.to_lowercase();
        let reply = self.retrieve(&format!("{}/v1/currencies/{}.json", iso_at, key));
        // get json hashmap, where the name is variable
        let base_rate = reply.json::<FreeRateEntry>().unwrap();
        let empty_rates = HashMap::new();
        let rates = base_rate.currencies.get(&key).unwrap_or(&empty_rates);
        ExchangeRate {
            base: base.to_string(),
            // keep KES and BDT
            rates: rates.iter().filter(|(k, _v)| {
                k == &"kes" || k == &"bdt"
            }).map(|(k, v)| {
                (k.to_uppercase(), *v)
            }).collect(),
        }
    }
}

impl RateProvider for FreeRateProvider {
    fn provider_name(&self) -> &'static str {
        "Free Exchange API"
    }

    fn latest(&self, base: &str) -> ExchangeRate {
        ExchangeRate::empty(base)
    }

    fn symbols(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn historical(
        &self,
        base: &str,
        from: &Date,
        to: &Date,
    ) -> HashMap<Date, ExchangeRate> {
        // iterate from to dates
        let mut rates = HashMap::new();
        let mut at = *from;
        while at <= *to {
            rates.insert(at, self.rates_from(base, &at));
            at = at.next_day().unwrap();
        }
        rates
    }
}
