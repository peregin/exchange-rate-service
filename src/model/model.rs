use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// structure for the exchangerate.host/frankfurter.app (same for both) response and exposed on rates endpoint as well
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExchangeRate {
    pub base: String,
    pub rates: HashMap<String, f32>,
}