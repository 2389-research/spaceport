//! `[usage]` display preferences; provider sign-in configuration is rejected.

use super::{model::Provider, registry};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UsageConfig {
    pub show: bool,
    pub topbar: bool,
    pub inline: bool,
    /// Provider ids whose missing reports may be shown as unavailable.
    pub show_providers: Vec<String>,
    /// Provider ids excluded from display.
    pub hide_providers: Vec<String>,
}

impl Default for UsageConfig {
    fn default() -> Self {
        Self {
            show: true,
            topbar: true,
            inline: true,
            show_providers: Vec::new(),
            hide_providers: Vec::new(),
        }
    }
}

impl UsageConfig {
    /// Drops display provider ids this build does not know and
    /// returns their key paths, so a config naming a provider from a newer
    /// build still loads.
    pub fn retain_known(&mut self) -> Vec<String> {
        let mut unknown = Vec::new();
        for (key, ids) in [
            ("show_providers", &mut self.show_providers),
            ("hide_providers", &mut self.hide_providers),
        ] {
            ids.retain(|id| {
                let known = registry::find(id).is_some();
                if !known {
                    unknown.push(format!("usage.{key}.{id}"));
                }
                known
            });
        }
        unknown
    }

    pub fn shown(&self, provider: Provider) -> bool {
        self.show_providers.iter().any(|id| id == provider.id())
    }

    pub fn hidden(&self, provider: Provider) -> bool {
        self.hide_providers.iter().any(|id| id == provider.id())
    }
}
