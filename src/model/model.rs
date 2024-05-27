use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

// 1./ structure for the exchangerate.host/frankfurter.app (same for both) response,
// 2./ exposed on rates endpoint as well
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ExchangeRate {
    #[schema(example = "CHF")]
    pub base: String,
    pub rates: HashMap<String, f32>,
}
