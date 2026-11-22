//! Reading settings from Herdr's `config.toml`, a file this app does not own.
//!
//! A newer Herdr may write values this build cannot read: an enum variant
//! added since, a key whose type changed. Each such value falls back to its
//! own default instead of failing the whole file, so one new setting cannot
//! stop the GUI following every other one. Malformed TOML still fails.
use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use std::collections::HashMap;

/// The value, or `None` when it has a shape this build does not read.
pub fn value<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = toml::Value::deserialize(deserializer)?;
    Ok(T::deserialize(value).ok())
}

/// The value, or `T::default()` when it cannot be read.
pub fn or_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    Ok(value(deserializer)?.unwrap_or_default())
}

/// For switches Herdr turns on by default.
pub fn or_true<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(value(deserializer)?.unwrap_or(true))
}

/// A table whose entries are read one by one: an entry this build cannot
/// read is dropped without losing the others.
pub fn entries<'de, D, T>(deserializer: D) -> Result<HashMap<String, T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let table: Option<HashMap<String, toml::Value>> = value(deserializer)?;
    Ok(table
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, value)| Some((key, T::deserialize(value).ok()?)))
        .collect())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
