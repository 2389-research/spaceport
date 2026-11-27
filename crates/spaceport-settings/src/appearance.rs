//! What every window draws with: one config and the theme resolved from it,
//! and the two sides a theme that follows the system is resolved into before
//! the first frame.
//!
//! Resolving a theme reads theme files, so callers run it on a worker. The app
//! publishes one of these to every window, and holding the pair in one value
//! is what stops a window drawing a theme its config did not name.

use crate::theme::ThemeName;
use crate::{Config, Result, Theme};

/// A config and the theme resolved from it, which only make sense together.
#[derive(Clone, Debug, Default)]
pub struct Appearance {
    pub config: Config,
    /// The theme `config` named, as resolved for one system appearance.
    pub theme: Theme,
}

impl Appearance {
    /// `config` and its theme for a system appearance, where `light` picks a
    /// side of a `light:…,dark:…` value. Reads theme files, so it runs off the
    /// UI thread.
    pub fn of(config: Config, light: bool) -> Result<Self> {
        let theme = config.theme(light)?;
        Ok(Self { config, theme })
    }
}

/// The two sides of a theme that follows the system, resolved before the first
/// frame so that picking one reads no file. A theme naming a single side has
/// only that one.
#[derive(Clone, Debug, Default)]
pub struct ThemeSides {
    dark: Appearance,
    light: Option<Theme>,
}

impl ThemeSides {
    /// `dark` resolved, plus the other side when its config follows the system.
    /// A light side that cannot load is logged and dropped, so the app still
    /// opens on the dark one.
    pub fn of(dark: Appearance) -> Self {
        let light = ThemeName::follows_system(&dark.config.theme)
            .then(|| dark.config.theme(true))
            .and_then(|theme| {
                theme
                    .inspect_err(|error| tracing::warn!(%error, "Could not load the light theme"))
                    .ok()
            });
        Self { dark, light }
    }

    /// The pair resolved for the dark side, which a launch reads before the
    /// system appearance is known.
    pub fn dark(&self) -> &Appearance {
        &self.dark
    }

    /// The pair a window draws with when the system appearance is `light`.
    pub fn select(self, light: bool) -> Appearance {
        match self.light {
            Some(theme) if light => Appearance { theme, ..self.dark },
            _ => self.dark,
        }
    }
}
