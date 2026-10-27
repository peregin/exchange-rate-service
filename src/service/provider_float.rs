use crate::route::model::ExchangeRate;
use crate::service::http;
use crate::service::provider::RateProvider;
use crate::service::serde_helpers::f32_from_number_or_string;
use async_trait::async_trait;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::Date;

pub struct FloatRateProvider {}

// internal response
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FloatRateEntry {
    pub code: String,
    pub name: String,
    #[serde(deserialize_with = "f32_from_number_or_string")]
    pub rate: f32,
}

impl FloatRateProvider {
    const HOST: &'static str = "https://www.floatrates.com";

    pub fn new() -> Self {
        FloatRateProvider {}
    }

    async fn retrieve(&self, base: &str) -> Vec<FloatRateEntry> {
        let url = format!(
            "{}/daily/{}.json",
            FloatRateProvider::HOST,
            base.to_lowercase()
        );
        match http::get_json::<HashMap<String, FloatRateEntry>>(&url).await {
            Ok(reply) => {
                info!("base={:#?}, {:#?} rates", base, reply.len());
                reply.into_values().collect()
            }
            Err(e) => {
                error!("FloatRates request failed for {}: {}", url, e);
                Vec::new()
            }
        }
    }
}

#[async_trait]
impl RateProvider for FloatRateProvider {
    fn provider_name(&self) -> &'static str {
        "floatrates.com"
    }

    // latest exchange rate

    async fn latest(&self, base: &str) -> ExchangeRate {
        let reply = self.retrieve(base).await;
        ExchangeRate {
            base: base.to_owned(),
            rates: reply.into_iter().map(|e| (e.code, e.rate)).collect(),
        }
    }

    async fn symbols(&self) -> HashMap<String, String> {
        self.retrieve("CHF")
            .await
            .into_iter()
            .map(|e| (e.code, e.name))
            .collect()
    }

    async fn historical(
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
    use super::*;
    use time::Month::November;

    #[actix_web::test]
    async fn test_historical_empty_response() {
        let provider = FloatRateProvider::new(); // Replace with your actual provider struct
        let base = "USD";
        let from = Date::from_calendar_date(2023, time::Month::January, 1).unwrap();
        let to = Date::from_calendar_date(2024, November, 11).unwrap();

        let result = provider.historical(base, &from, &to).await;

        assert!(result.is_empty());
    }

    #[actix_web::test]
    async fn test_historical_date_range() {
        let provider = FloatRateProvider::new();
        let base = "EUR";
        let from = Date::from_calendar_date(2024, November, 11).unwrap();
        let to = from + time::Duration::days(10);

        let result = provider.historical(base, &from, &to).await;

        assert!(result.is_empty());
    }

    #[test]
    fn test_float_rate_entry_accepts_string_rate() {
        let entry: FloatRateEntry =
            serde_json::from_str(r#"{"code":"KES","name":"Kenyan Shilling","rate":"1.78073471"}"#)
                .unwrap();

        assert_eq!(entry.rate, 1.7807347);
    }
}
