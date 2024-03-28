use serde::{de, Deserialize, Deserializer};

// Function to deserialize a value as either a string or a floating-point number
pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrFloat {
        String(String),
        Float(f64),
    }

    match StringOrFloat::deserialize(deserializer)? {
        StringOrFloat::String(s) => s.parse().map_err(de::Error::custom),
        StringOrFloat::Float(i) => Ok(i),
    }
}
