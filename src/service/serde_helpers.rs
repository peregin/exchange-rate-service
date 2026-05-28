use serde::de::{self, Deserializer};
use serde::Deserialize;

pub(crate) fn f32_from_number_or_string<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(number) => number
            .as_f64()
            .map(|value| value as f32)
            .ok_or_else(|| de::Error::custom("expected a finite f32-compatible number")),
        serde_json::Value::String(value) => value.parse::<f32>().map_err(de::Error::custom),
        _ => Err(de::Error::custom(
            "expected exchange rate as number or numeric string",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::f32_from_number_or_string;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Rate {
        #[serde(deserialize_with = "f32_from_number_or_string")]
        value: f32,
    }

    #[test]
    fn test_f32_from_number_or_string_accepts_number() {
        let rate: Rate = serde_json::from_str(r#"{"value":1.78073471}"#).unwrap();

        assert_eq!(rate.value, 1.7807347);
    }

    #[test]
    fn test_f32_from_number_or_string_accepts_string() {
        let rate: Rate = serde_json::from_str(r#"{"value":"1.78073471"}"#).unwrap();

        assert_eq!(rate.value, 1.7807347);
    }

    #[test]
    fn test_f32_from_number_or_string_rejects_non_numeric_string() {
        let result = serde_json::from_str::<Rate>(r#"{"value":"not-a-rate"}"#);

        assert!(result.is_err());
    }
}
