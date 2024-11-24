use cached::proc_macro::cached;
use log::info;
use std::collections::HashMap;
use std::sync::LazyLock;
use time::Date;

use crate::route::model::ExchangeRate;
use crate::service::provider_ecb::EcbRateProvider;
use crate::service::provider_float::FloatRateProvider;
use crate::service::provider_free::FreeRateProvider;

// generic contract what needs to be implemented by any rate provider
pub trait RateProvider: Sync + Send {
    fn provider_name(&self) -> &str;

    fn latest(&self, base: &str) -> ExchangeRate;

    // iso3 -> description
    fn symbols(&self) -> HashMap<String, String>;

    fn historical(&self, base: &str, from: &Date, to: &Date) -> HashMap<Date, ExchangeRate>;
}

type Providers = Vec<Box<dyn RateProvider>>;

fn get_providers() -> &'static Providers {
    static PROVIDERS: LazyLock<Providers, fn() -> Providers> = LazyLock::new(|| {
        // sequence is important, the latter will override the same currencies
        // ECB rates override float rates
        let providers: Providers = vec![
            Box::new(EcbRateProvider::new()),
            Box::new(FloatRateProvider::new()),
            Box::new(FreeRateProvider::new()),
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

#[cached(time = 86400)]
pub fn count_providers() -> usize {
    get_providers().len()
}

#[cached(time = 3600)]
pub fn rates_of(base: String) -> ExchangeRate {
    rates_of_with(&base, get_providers)
}

fn rates_of_with<F>(base: &str, providers_fn: F) -> ExchangeRate
where
    F: Fn() -> &'static Providers,
{
    let rates = providers_fn().iter().map(|p| p.latest(base));
    // merge with priority (ECB rates overrides floating rates)
    rates.fold(ExchangeRate::empty(base), |acc, current| current.chain(acc))
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
pub fn historical_rates_of(base: String, from: Date, to: Date) -> HashMap<Date, ExchangeRate> {
    info!("historical_rates_of: {} {} {}", base, from, to);
    historical_rates_of_with(&base, from, to, get_providers)
}

fn historical_rates_of_with<F>(
    base: &str,
    from: Date,
    to: Date,
    providers_fn: F,
) -> HashMap<Date, ExchangeRate>
where
    F: Fn() -> &'static Providers,
{
    let rates = providers_fn()
        .iter()
        .flat_map(|p| p.historical(base, &from, &to).into_iter());
    // merge with priority (ECB rates overrides floating rates)
    rates.fold(HashMap::new(), |mut acc, (date, current)| {
        if let Some(existing) = acc.get_mut(&date) {
            *existing = current.chain(existing.clone());
        } else {
            acc.insert(date, current);
        }
        acc
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::ops::Add;
    use std::sync::OnceLock;
    use time::Month::November;
    use time::{Duration, Month};

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

        fn historical(&self, base: &str, from: &Date, to: &Date) -> HashMap<Date, ExchangeRate> {
            // days between from and to
            let days = to.to_julian_day() - from.to_julian_day();
            // iterate between from until to and create ExchangeRate for each day
            let mut rates = HashMap::new();
            for i in 0..=days {
                let date = from.add(Duration::days(i as i64));
                let exchange_rate = ExchangeRate {
                    base: base.to_string(),
                    // add 1 to each rate to make it different from the base
                    // and make it easier to test
                    // 1.1, 1.2, 1.3, ...
                    rates: self
                        .rates
                        .iter()
                        .map(|(k, v)| (k.clone(), v + i as f32 + 1.0))
                        .collect(),
                };
                let next: Date = Date::from_calendar_date(
                    date.year() as i32,
                    Month::try_from(date.month() as u8).unwrap(),
                    date.day() as u8,
                )
                .unwrap();
                rates.insert(next, exchange_rate);
            }
            rates
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

        let result = rates_of_with("EUR", || MOCK_PROVIDERS.get().unwrap());

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
        MOCK_PROVIDERS.get_or_init(|| vec![Box::new(ecb_provider), Box::new(floating_provider)]);

        let result = rates_of_with("EUR", || MOCK_PROVIDERS.get().unwrap());

        assert_eq!(result.base, "EUR");
        assert_eq!(result.rates.len(), 3);
        assert_eq!(result.rates.get("USD"), Some(&1.1)); // ECB rate
        assert_eq!(result.rates.get("GBP"), Some(&0.85));
        assert_eq!(result.rates.get("JPY"), Some(&130.0));
    }

    #[test]
    fn test_rates_of_empty_providers() {
        static TEST_PROVIDERS: Providers = vec![];

        let result = rates_of_with("EUR", || &TEST_PROVIDERS);

        assert_eq!(result.base, "EUR");
        assert!(result.rates.is_empty());
    }

    #[test]
    fn test_historical_rates_with_multiple_providers_and_priority() {
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
        MOCK_PROVIDERS.get_or_init(|| vec![Box::new(ecb_provider), Box::new(floating_provider)]);

        let from = Date::from_calendar_date(2024, November, 12).unwrap();
        let to = from.add(Duration::days(3 as i64));
        let result = historical_rates_of_with("EUR", from, to, || MOCK_PROVIDERS.get().unwrap());

        //println!("{:#?}", result);
        assert_eq!(result.len(), 4);
        let day1 = result.get(&from).unwrap();
        assert_eq!(day1.base, "EUR");
        assert_eq!(day1.rates.len(), 3);
        assert_eq!(day1.rates.get("USD"), Some(&2.1)); // ECB rate
        assert_eq!(day1.rates.get("GBP"), Some(&1.85));
        assert_eq!(day1.rates.get("JPY"), Some(&131.0));
        let day4 = result.get(&to).unwrap();
        assert_eq!(day4.base, "EUR");
        assert_eq!(day4.rates.len(), 3);
        assert_eq!(day4.rates.get("USD"), Some(&5.1)); // ECB rate
        assert_eq!(day4.rates.get("GBP"), Some(&4.85));
        assert_eq!(day4.rates.get("JPY"), Some(&134.0));
    }

    #[test]
    fn test_historical_rates_with_empty_multiple_providers() {
        let ecb_rates = HashMap::new();
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
        MOCK_PROVIDERS.get_or_init(|| vec![Box::new(ecb_provider), Box::new(floating_provider)]);

        let from = Date::from_calendar_date(2024, November, 12).unwrap();
        let to = from.add(Duration::days(2 as i64));
        let result = historical_rates_of_with("EUR", from, to, || MOCK_PROVIDERS.get().unwrap());

        println!("{:#?}", result);
        assert_eq!(result.len(), 3);
        let day1 = result.get(&from).unwrap();
        assert_eq!(day1.base, "EUR");
        assert_eq!(day1.rates.len(), 2);
        assert_eq!(day1.rates.get("USD"), Some(&2.2)); // ECB rate
        assert!(!day1.rates.contains_key("GBP"));
        assert_eq!(day1.rates.get("JPY"), Some(&131.0));
        let day3 = result.get(&to).unwrap();
        assert_eq!(day3.base, "EUR");
        assert_eq!(day3.rates.len(), 2);
        assert_eq!(day3.rates.get("USD"), Some(&4.2)); // ECB rate
        assert!(!day3.rates.contains_key("GBP"));
        assert_eq!(day3.rates.get("JPY"), Some(&133.0));
    }
}
