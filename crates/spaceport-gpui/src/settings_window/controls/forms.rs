//! Form pieces used by chat environment and appearance controls: a titled
//! block under the shared section heading and a labelled switch row, from
//! `spaceport-settings-window`'s kit, with this window's rows around them.
// i18n: migrated
use super::*;
use spaceport_settings_window::forms;

impl SettingsWindow {
    /// A block of form content under a section heading; no card frame.
    pub(in crate::settings_window) fn control_card(&self, title: &str) -> Div {
        forms::control_card(&self.theme, title)
    }

    /// A clickable row with `label` and a switch; the caller adds `.on_click`
    /// when `enabled`.
    pub(in crate::settings_window) fn control_switch(
        &self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        checked: bool,
        enabled: bool,
    ) -> Stateful<Div> {
        forms::control_switch(&self.theme, id, label, checked, enabled)
    }
}
