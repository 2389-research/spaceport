//! The retained `[container]` table; saved rows still use the engine key.
//! Old provisioning and auto-stop values parse without changing user files.
#[cfg(feature = "container")]
use crate::Result;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ContainerConfig {
    // keep-sorted start
    /// Old auto-stop value, preserved but not used by the client.
    pub auto_stop: Option<String>,
    #[serde(deserialize_with = "text")]
    pub cpus: Option<String>,
    /// `docker` or `podman`; the first one installed when unset.
    pub engine: Option<String>,
    /// The image new containers start from, such as `ubuntu:24.04`.
    pub image: Option<String>,
    /// The most containers kept as devices at once; 8 by default.
    #[serde(deserialize_with = "text")]
    pub max_containers: Option<String>,
    #[serde(deserialize_with = "text")]
    pub memory: Option<String>,
    #[serde(deserialize_with = "text")]
    pub pids_limit: Option<String>,
    // keep-sorted end
}

/// A string, or a number read as its text.
fn text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Text {
        Integer(i64),
        Float(f64),
        String(String),
    }
    Ok(Some(match Text::deserialize(deserializer)? {
        Text::Integer(value) => value.to_string(),
        Text::Float(value) => value.to_string(),
        Text::String(value) => value,
    }))
}

#[cfg(feature = "container")]
impl ContainerConfig {
    /// The validated settings new containers are made with.
    pub fn settings(&self) -> Result<crate::container::Settings> {
        Ok(crate::container::Settings::resolve(self)?)
    }
}

/// The `[container]` keys the Settings window edits. An empty value removes
/// its key.
#[cfg(feature = "container")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContainerFields {
    pub engine: String,
}

#[cfg(feature = "container")]
impl ContainerFields {
    pub fn from_config(config: &ContainerConfig) -> Self {
        let text = |value: &Option<String>| value.clone().unwrap_or_default();
        Self {
            engine: text(&config.engine),
        }
    }

    pub fn entries(&self) -> [(&'static str, &str); 1] {
        [("engine", &self.engine)]
    }

    /// The checks loading applies, so a save cannot leave a config that then
    /// fails to load.
    pub fn validate(&self, existing: &ContainerConfig) -> Result<()> {
        let value = |text: &str| {
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_owned())
        };
        ContainerConfig {
            engine: value(&self.engine),
            ..existing.clone()
        }
        .settings()
        .map(drop)
    }
}
