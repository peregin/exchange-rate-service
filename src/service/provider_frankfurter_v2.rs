use crate::route::model::ExchangeRate;
use crate::service::provider::RateProvider;
use async_trait::async_trait;
use log::{error, info};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use time::format_description::well_known::Iso8601;
use time::Date;

pub struct FrankfurterV2RateProvider {}

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FrankfurterV2RateEntry {
    date: String,
    base: String,
    quote: String,
    rate: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FrankfurterV2Currency {
    iso_code: String,
    name: String,
}

impl FrankfurterV2RateProvider {
    const HOST: &'static str = "https://api.frankfurter.dev/v2";

    pub fn new() -> Self {
        FrankfurterV2RateProvider {}
    }

    async fn retrieve<T>(&self, path: &str) -> T
    where
        T: DeserializeOwned + Default,
    {
        let url = format!("{}/{}", FrankfurterV2RateProvider::HOST, path);
        match HTTP_CLIENT
            .get(&url)
            .header("User-Agent", "actix-web")
            .header("Content-Type", "application/json")
            .send()
            .await
        {
            Ok(reply) => match reply.error_for_status() {
                Ok(reply) => reply.json::<T>().await.unwrap_or_else(|e| {
                    error!(
                        "Failed to parse Frankfurter v2 response from {}: {}",
                        url, e
                    );
                    T::default()
                }),
                Err(e) => {
                    error!("Frankfurter v2 request failed for {}: {}", url, e);
                    T::default()
                }
            },
            Err(e) => {
                error!("Frankfurter v2 request failed for {}: {}", url, e);
                T::default()
            }
        }
    }

    fn rows_to_exchange_rate(base: &str, rows: Vec<FrankfurterV2RateEntry>) -> ExchangeRate {
        ExchangeRate {
            base: base.to_string(),
            rates: rows
                .into_iter()
                .map(|entry| (entry.quote, entry.rate))
                .collect(),
        }
    }

    fn rows_to_history(
        base: &str,
        rows: Vec<FrankfurterV2RateEntry>,
    ) -> HashMap<Date, ExchangeRate> {
        let format = Iso8601::DATE;
        let mut history: HashMap<Date, ExchangeRate> = HashMap::new();

        for entry in rows {
            match Date::parse(&entry.date, &format) {
                Ok(date) => {
                    history
                        .entry(date)
                        .or_insert_with(|| ExchangeRate::empty(base))
                        .rates
                        .insert(entry.quote, entry.rate);
                }
                Err(e) => {
                    error!("Failed to parse Frankfurter v2 date {}: {}", entry.date, e);
                }
            }
        }

        history
    }
}

#[async_trait]
impl RateProvider for FrankfurterV2RateProvider {
    fn provider_name(&self) -> &'static str {
        "Frankfurter v2"
    }

    async fn latest(&self, base: &str) -> ExchangeRate {
        let rows = self
            .retrieve::<Vec<FrankfurterV2RateEntry>>(&format!("rates?base={}", base))
            .await;
        info!("base={:#?}, {:#?} Frankfurter v2 rates", base, rows.len());
        Self::rows_to_exchange_rate(base, rows)
    }

    async fn symbols(&self) -> HashMap<String, String> {
        self.retrieve::<Vec<FrankfurterV2Currency>>("currencies")
            .await
            .into_iter()
            .map(|entry| (entry.iso_code, entry.name))
            .collect()
    }

    async fn historical(&self, base: &str, from: &Date, to: &Date) -> HashMap<Date, ExchangeRate> {
        let format = Iso8601::DATE;
        let iso_from = from.format(&format).unwrap();
        let iso_to = to.format(&format).unwrap();
        let rows = self
            .retrieve::<Vec<FrankfurterV2RateEntry>>(&format!(
                "rates?base={}&from={}&to={}",
                base, iso_from, iso_to
            ))
            .await;
        Self::rows_to_history(base, rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn test_rows_to_exchange_rate() {
        let rows = vec![
            FrankfurterV2RateEntry {
                date: "2024-01-01".to_string(),
                base: "CHF".to_string(),
                quote: "NPR".to_string(),
                rate: 158.18,
            },
            FrankfurterV2RateEntry {
                date: "2024-01-01".to_string(),
                base: "CHF".to_string(),
                quote: "UGX".to_string(),
                rate: 4603.61,
            },
        ];

        let rates = FrankfurterV2RateProvider::rows_to_exchange_rate("CHF", rows);

        assert_eq!(rates.base, "CHF");
        assert_eq!(rates.rates.len(), 2);
        assert_eq!(rates.rates.get("NPR"), Some(&158.18));
        assert_eq!(rates.rates.get("UGX"), Some(&4603.61));
    }

    #[test]
    fn test_rows_to_history_groups_rates_by_date() {
        let rows = vec![
            FrankfurterV2RateEntry {
                date: "2024-01-01".to_string(),
                base: "CHF".to_string(),
                quote: "NPR".to_string(),
                rate: 158.18,
            },
            FrankfurterV2RateEntry {
                date: "2024-01-01".to_string(),
                base: "CHF".to_string(),
                quote: "UGX".to_string(),
                rate: 4603.61,
            },
            FrankfurterV2RateEntry {
                date: "2024-01-02".to_string(),
                base: "CHF".to_string(),
                quote: "NPR".to_string(),
                rate: 157.76,
            },
        ];

        let history = FrankfurterV2RateProvider::rows_to_history("CHF", rows);

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(&date!(2024 - 01 - 01)).unwrap().base, "CHF");
        assert_eq!(
            history
                .get(&date!(2024 - 01 - 01))
                .unwrap()
                .rates
                .get("NPR"),
            Some(&158.18)
        );
        assert_eq!(
            history
                .get(&date!(2024 - 01 - 01))
                .unwrap()
                .rates
                .get("UGX"),
            Some(&4603.61)
        );
        assert_eq!(
            history
                .get(&date!(2024 - 01 - 02))
                .unwrap()
                .rates
                .get("NPR"),
            Some(&157.76)
        );
    }
}
