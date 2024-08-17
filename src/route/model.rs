use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

// models exposed to the public via api, be careful when changing it (and adapt up-streams)
//
// structure used for the frankfurter.app and exchangerate.host (same for both) response
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ExchangeRate {
    #[schema(example = "CHF")]
    pub base: String,
    #[schema(example = r#"{"USD": 1.0, "EUR": 0.9, "JPY": 110.5}"#)]
    pub rates: HashMap<String, f32>,
}

impl ExchangeRate {

    pub fn chain(&self, that: ExchangeRate) -> ExchangeRate {
        ExchangeRate {
            base: that.base,
            rates: self.rates.clone().into_iter().chain(that.rates).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ExchangeRate;
    use std::collections::HashMap;

    #[test]
    fn test_chain() {
        let mut rates1 = HashMap::new();
        rates1.insert("USD".to_string(), 1.0);
        rates1.insert("EUR".to_string(), 0.9);

        let mut rates2 = HashMap::new();
        rates2.insert("GBP".to_string(), 0.8);
        rates2.insert("JPY".to_string(), 120.0);
        rates2.insert("USD".to_string(), 1.1);

        let exchange_rate1 = ExchangeRate {
            base: "USD".to_string(),
            rates: rates1,
        };

        let exchange_rate2 = ExchangeRate {
            base: "USD".to_string(),
            rates: rates2,
        };

        let chained = exchange_rate1.chain(exchange_rate2);

        assert_eq!(chained.base, "USD");
        assert_eq!(chained.rates.len(), 4);
        assert_eq!(chained.rates.get("USD"), Some(&1.1)); // second is overriding
        assert_eq!(chained.rates.get("EUR"), Some(&0.9));
        assert_eq!(chained.rates.get("GBP"), Some(&0.8));
        assert_eq!(chained.rates.get("JPY"), Some(&120.0));
    }
}

