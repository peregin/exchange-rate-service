use crate::route::model::ExchangeRate;
use crate::service::http;
use crate::service::provider::RateProvider;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::format_description::well_known::Iso8601;
use time::Date;

pub struct FreeRateProvider {}

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
        FreeRateProvider {}
    }

    async fn rates_from(&self, base: &str, at: &Date) -> ExchangeRate {
        let format = Iso8601::DATE;
        let iso_at = at.format(&format).unwrap();
        let key = base.to_lowercase();
        let url = format!("{}@{}/v1/currencies/{}.json", Self::HOST, iso_at, key);
        let base_rate = match http::get_json::<FreeRateEntry>(&url).await {
            Ok(reply) => reply,
            Err(e) => {
                log::error!("Free Exchange API request failed for {}: {}", url, e);
                return ExchangeRate::empty(base);
            }
        };
        let empty_rates = HashMap::new();
        let rates: &HashMap<String, f32> = base_rate.currencies.get(&key).unwrap_or(&empty_rates);
        ExchangeRate {
            base: base.to_string(),
            // keep KES and BDT
            rates: rates
                .iter()
                .filter(|(k, _v)| k == &"kes" || k == &"bdt")
                .map(|(k, v)| (k.to_uppercase(), *v))
                .collect(),
        }
    }

    async fn rates_between(
        &self,
        base: &str,
        from: &Date,
        to: &Date,
    ) -> HashMap<Date, ExchangeRate> {
        // create a vec of dates from to
        let mut dates = Vec::new();
        let mut current = *from;
        while current <= *to {
            dates.push(current);
            current = current.next_day().unwrap();
        }

        stream::iter(dates)
            .map(|day| async move {
                let rate = self.rates_from(base, &day).await;
                (day, rate)
            })
            .buffer_unordered(10) // Process up to 10 requests concurrently
            .collect()
            .await
    }
}

#[async_trait]
impl RateProvider for FreeRateProvider {
    fn provider_name(&self) -> &'static str {
        "Free Exchange API"
    }

    async fn latest(&self, base: &str) -> ExchangeRate {
        ExchangeRate::empty(base)
    }

    async fn symbols(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    async fn historical(&self, base: &str, from: &Date, to: &Date) -> HashMap<Date, ExchangeRate> {
        self.rates_between(base, from, to).await
    }
}
