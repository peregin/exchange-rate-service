use crate::route::model::ExchangeRate;
use crate::service::provider::{ProviderFuture, RateProvider};
use log::info;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;
use time::format_description::well_known::Iso8601;
use time::Date;

pub struct EcbRateProvider {}

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

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
        EcbRateProvider {}
    }

    async fn retrieve(&self, path: &str) -> Response {
        HTTP_CLIENT
            .get(format!("{}/{}", EcbRateProvider::HOST, path))
            .header("User-Agent", "actix-web")
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap()
    }
}

impl RateProvider for EcbRateProvider {
    fn provider_name(&self) -> &'static str {
        "European Central Bank"
    }

    fn latest<'a>(&'a self, base: &'a str) -> ProviderFuture<'a, ExchangeRate> {
        Box::pin(async move {
            let reply = self.retrieve(&format!("latest?from={}", base)).await;
            let reply = reply.json::<ExchangeRate>().await.unwrap();
            info!("base={:#?}, {:#?} rates", base, reply.rates.keys().len());
            reply
        })
    }

    fn symbols(&self) -> ProviderFuture<'_, HashMap<String, String>> {
        Box::pin(async move {
            let reply = self.retrieve("currencies").await;
            reply.json::<HashMap<String, String>>().await.unwrap()
        })
    }

    fn historical<'a>(
        &'a self,
        base: &'a str,
        from: &'a Date,
        to: &'a Date,
    ) -> ProviderFuture<'a, HashMap<Date, ExchangeRate>> {
        Box::pin(async move {
            let format = Iso8601::DATE;
            let iso_from = from.format(&format).unwrap();
            let iso_to = to.format(&format).unwrap();
            let reply = self
                .retrieve(&format!("{}..{}?from={}", iso_from, iso_to, base))
                .await;

            let rate_history: EcbRateHistory = reply.json::<EcbRateHistory>().await.unwrap();
            //println!("rate_history={:#?}", rate_history);
            rate_history
                .rates
                .into_iter()
                .map(|(date, rates)| {
                    let iso_date = Date::parse(&date, &format).unwrap();
                    let exchange_rate = ExchangeRate {
                        base: base.to_string(),
                        rates,
                    };
                    (iso_date, exchange_rate)
                })
                .collect()
        })
    }
}
