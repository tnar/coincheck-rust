use serde::{de, Deserialize, Deserializer};

// Function to deserialize a value as a vector of pairs of floating-point numbers
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[f64; 2]>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrFloat {
        String(String),
        Float(f64),
    }

    // Deserialize the value as a vector of vectors of StringOrFloat values
    let v = Vec::<Vec<StringOrFloat>>::deserialize(deserializer)?;

    // Convert the deserialized value into a vector of pairs of floating-point numbers
    v.into_iter()
        .map(|x| {
            if x.len() != 2 {
                return Err(de::Error::custom("expected array of length 2"));
            }
            Ok([
                match &x[0] {
                    StringOrFloat::String(s) => s.parse().map_err(de::Error::custom)?,
                    StringOrFloat::Float(i) => *i,
                },
                match &x[1] {
                    StringOrFloat::String(s) => s.parse().map_err(de::Error::custom)?,
                    StringOrFloat::Float(i) => *i,
                },
            ])
        })
        .collect()
}
