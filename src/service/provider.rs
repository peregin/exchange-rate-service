use cached::proc_macro::cached;
use chrono::{DateTime, Utc};
use log::info;
use std::collections::HashMap;
use std::sync::LazyLock;
use time::Date;

use crate::route::model::ExchangeRate;
use crate::service::provider_ecb::EcbRateProvider;
use crate::service::provider_float::FloatRateProvider;

// generic contract what needs to be implemented by any rate provider
pub trait RateProvider: Sync + Send {
    fn provider_name(&self) -> &str;

    fn latest(&self, base: &str) -> ExchangeRate;

    // iso3 -> description
    fn symbols(&self) -> HashMap<String, String>;

    fn historical(
        &self,
        base: &str,
        from: &DateTime<Utc>,
        to: &DateTime<Utc>,
    ) -> HashMap<Date, ExchangeRate>;
}

type Providers = Vec<Box<dyn RateProvider>>;

fn get_providers() -> &'static Providers {
    static PROVIDERS: LazyLock<Providers, fn() -> Providers> = LazyLock::new(|| {
        // sequence is important, the latter will override the same currencies
        // ECB rates override float rates
        let providers: Providers = vec![
            Box::new(EcbRateProvider::new()),
            Box::new(FloatRateProvider::new()),
        ];
        info!(
            "providers: {:?}",
            providers
                .iter()
                .map(|p| p.provider_name())
                .collect::<Vec<&str>>()
        );
        providers
    });
    &PROVIDERS
}

#[cached(time = 3600)]
pub fn rates_of(base: String) -> ExchangeRate {
    rates_of_with(base, get_providers)
}

fn rates_of_with<F>(base: String, providers_fn: F) -> ExchangeRate
where
    F: Fn() -> &'static Providers,
{
    let rates = providers_fn()
        .iter()
        .map(|p| p.latest(&base));
    // merge with priority (ECB rates overrides floating rates)
    rates.fold(ExchangeRate::empty(&base), |acc, current| current.chain(acc))
}

// map of ISO3 code -> description
#[cached(time = 3600)]
pub fn symbols() -> HashMap<String, String> {
    get_providers()
        .iter()
        .flat_map(|p| p.symbols().into_iter())
        .collect()
}

#[cached(time = 3600)]
pub fn historical_rates_of(
    base: String,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> HashMap<Date, ExchangeRate> {
    EcbRateProvider::new().historical(&base, &from, &to)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::OnceLock;

    // Mock provider for testing
    struct MockProvider {
        name: String,
        rates: HashMap<String, f32>,
    }

    #[allow(unused_variables)]
    impl RateProvider for MockProvider {
        fn provider_name(&self) -> &str {
            &self.name
        }

        fn latest(&self, base: &str) -> ExchangeRate {
            ExchangeRate {
                base: base.to_string(),
                rates: self.rates.clone(),
            }
        }

        fn symbols(&self) -> HashMap<String, String> {
            todo!()
        }

        fn historical(&self, base: &str, from: &DateTime<Utc>, to: &DateTime<Utc>) -> HashMap<Date, ExchangeRate> {
            todo!()
        }
    }

    #[test]
    fn test_rates_of_single_provider() {
        let mut rates = HashMap::new();
        rates.insert("USD".to_string(), 1.1);
        rates.insert("GBP".to_string(), 0.85);
        let mock_provider = MockProvider {
            name: "Test Provider".to_string(),
            rates,
        };
        static MOCK_PROVIDERS: OnceLock<Providers> = OnceLock::new();
        MOCK_PROVIDERS.get_or_init(|| vec![Box::new(mock_provider)]);

        let result = rates_of_with("EUR".to_string(), || MOCK_PROVIDERS.get().unwrap());

        assert_eq!(result.base, "EUR");
        assert_eq!(result.rates.len(), 2);
        assert_eq!(result.rates.get("USD"), Some(&1.1));
        assert_eq!(result.rates.get("GBP"), Some(&0.85));
    }

    #[test]
    fn test_rates_of_multiple_providers_with_priority() {
        // Arrange
        let mut ecb_rates = HashMap::new();
        ecb_rates.insert("USD".to_string(), 1.1);
        ecb_rates.insert("GBP".to_string(), 0.85);

        let mut floating_rates = HashMap::new();
        floating_rates.insert("USD".to_string(), 1.2); // Should be overridden by ECB
        floating_rates.insert("JPY".to_string(), 130.0); // Should be included

        let ecb_provider = MockProvider {
            name: "ECB".to_string(),
            rates: ecb_rates,
        };
        let floating_provider = MockProvider {
            name: "Floating".to_string(),
            rates: floating_rates,
        };
        static MOCK_PROVIDERS: OnceLock<Providers> = OnceLock::new();
        // use the same order as in the real providers
        MOCK_PROVIDERS.get_or_init(|| vec![
            Box::new(ecb_provider),
            Box::new(floating_provider),
        ]);

        let result = rates_of_with("EUR".to_string(), || MOCK_PROVIDERS.get().unwrap());

        assert_eq!(result.base, "EUR");
        assert_eq!(result.rates.len(), 3);
        assert_eq!(result.rates.get("USD"), Some(&1.1)); // ECB rate
        assert_eq!(result.rates.get("GBP"), Some(&0.85));
        assert_eq!(result.rates.get("JPY"), Some(&130.0));
    }

    #[test]
    fn test_rates_of_empty_providers() {
        static TEST_PROVIDERS: Providers = vec![];

        let result = rates_of_with("EUR".to_string(), || &TEST_PROVIDERS);

        assert_eq!(result.base, "EUR");
        assert!(result.rates.is_empty());
    }
}

