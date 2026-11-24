//! What one page draws, prepared by the window.
//!
//! A page shows the settings being edited, the theme it paints with and
//! Herdr's shared settings, and leaves out the rows this build cannot offer.
//! It reads them here instead of through the window, so a renderer draws from
//! values it was handed and a page can move below the app with them.

use super::{SettingsWindow, catalog, ui};
use crate::{
    config::{Config, Theme, ThemeName},
    herdr_settings,
    i18n::fl,
};
use gpui::App;

/// The values one page draws, as the window reads them for one render.
pub(in crate::settings_window) struct PageState<'a> {
    /// The settings being edited: what the files hold, with this window's
    /// unsaved drafts over it.
    pub(in crate::settings_window) config: &'a Config,
    /// Whether this build shows herdr-core features, which the Core opt-in
    /// freezes for the whole app session.
    pub(in crate::settings_window) core_enabled: bool,
    /// The theme value the next choice builds on: an unsaved draft, else the
    /// saved one.
    pub(in crate::settings_window) drafted_theme: &'a str,
    /// Herdr's shared settings, or none when this build did not load them.
    pub(in crate::settings_window) shared: Option<&'a herdr_settings::Settings>,
    /// The theme every row of the page paints with.
    pub(in crate::settings_window) theme: &'a Theme,
}

impl<'a> PageState<'a> {
    /// The values `window` draws in one render of a page.
    pub(in crate::settings_window) fn of(window: &'a SettingsWindow, cx: &App) -> Self {
        Self {
            config: &window.config,
            core_enabled: crate::core_feature::enabled(cx),
            drafted_theme: window.drafted_theme(),
            shared: window.shared.as_ref(),
            theme: &window.theme,
        }
    }

    /// Whether the page renders the row needing `availability` right now.
    pub(in crate::settings_window) fn available(
        &self,
        availability: catalog::Availability,
    ) -> bool {
        match availability {
            catalog::Availability::Always => true,
            catalog::Availability::AcpChats => self.config.features.acp_chats,
            catalog::Availability::Core => self.core_enabled,
            catalog::Availability::FollowingSystemTheme => {
                ThemeName::follows_system(self.drafted_theme)
            }
            catalog::Availability::LimitedChatWidth => {
                self.config.features.acp_chats && self.config.chat.max_width != 0
            }
            catalog::Availability::Linux => cfg!(target_os = "linux"),
            catalog::Availability::Windows => cfg!(windows),
        }
    }

    /// Why rows that edit Herdr's shared file are disabled, if they are.
    pub(in crate::settings_window) fn shared_gate(&self) -> Option<String> {
        shared_gate(self.shared)
    }

    /// The row, disabled with that reason where it cannot be edited.
    pub(in crate::settings_window) fn gate_shared(&self, row: ui::Row) -> ui::Row {
        match self.shared_gate() {
            Some(reason) => row.disabled(reason),
            None => row,
        }
    }
}

/// Why rows that edit Herdr's shared file are disabled, if they are: this
/// build cannot write that file, or it has not been read.
pub(in crate::settings_window) fn shared_gate(
    shared: Option<&herdr_settings::Settings>,
) -> Option<String> {
    if !cfg!(unix) {
        Some(fl!("settings-shared-read-only"))
    } else if shared.is_none() {
        Some(fl!("settings-shared-unavailable"))
    } else {
        None
    }
}
