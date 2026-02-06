use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};

pub fn deserialize_opt_trimmed_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
}

pub fn deserialize_trimmed_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim().to_string();
    if s.is_empty() {
        return Err(D::Error::invalid_value(
            Unexpected::Str(""),
            &"non-empty string",
        ));
    }
    Ok(s)
}
