use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

// models exposed to the public via api, be careful when changing it (and adapt upstreams)

// structure used for the frankfurter.app and exchangerate.host (same for both) response
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ExchangeRate {
    #[schema(example = "CHF")]
    pub base: String,
    #[schema(example = r#"{"USD": 1.0, "EUR": 0.9, "JPY": 110.5}"#)]
    pub rates: HashMap<String, f32>,
}
