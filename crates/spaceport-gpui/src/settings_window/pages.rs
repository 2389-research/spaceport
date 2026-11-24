//! The behaviour pages: one module per Settings page, each rebuilt from the
//! shared row and control builders in `ui`.
mod configuration;
mod fonts;
mod general;
mod indicators;
mod language;
pub(in crate::settings_window) mod notifications;
mod sidebar;
mod status_bar;
mod tabs;
mod theme;
mod window;

use super::{PageState, SettingsWindow, catalog, inputs::DropdownField, ui};
use crate::{
    config::Theme,
    i18n::{self, fl},
};
use gpui::{prelude::*, *};
pub(super) use language::language_choices;
pub(super) use spaceport_settings::fields::{
    CLIPBOARD_POSITIONS, Spacing, TOAST_POSITIONS, padding_ids, position_message,
};

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub(in crate::settings_window) use configuration::open_path;

/// The text on an override dropdown: the local choice, or what Herdr says.
pub(super) fn override_label(shared_value: &str, local: Option<&str>) -> String {
    match local {
        Some(choice) => choice.to_owned(),
        None => fl!("settings-use-herdr", value = shared_value),
    }
}

impl SettingsWindow {
    /// A dropdown whose first entry follows Herdr and whose other entries are
    /// local overrides. `local` is the index into `choices` of the current one.
    fn override_dropdown(
        &self,
        theme: &Theme,
        field: DropdownField,
        shared_value: &str,
        choices: &[String],
        local: Option<usize>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let label = override_label(
            shared_value,
            local.and_then(|i| choices.get(i)).map(String::as_str),
        );
        let options: Vec<SharedString> = std::iter::once(override_label(shared_value, None))
            .chain(choices.iter().cloned())
            .map(SharedString::from)
            .collect();
        let selected = local.map_or(0, |index| index + 1);
        let view = cx.entity().downgrade();
        let (opener, picker, dismisser) = (view.clone(), view.clone(), view);
        ui::dropdown(
            theme,
            field.selector(),
            label,
            true,
            true,
            self.dropdown.as_ref().filter(|open| open.field == field),
            move |window, cx| {
                let options = options.clone();
                let _ = opener.update(cx, |this, cx| {
                    this.open_dropdown(field, options, selected, window, cx)
                });
            },
            move |index, window, cx| {
                let _ = picker.update(cx, |this, cx| {
                    this.pick_dropdown(field, index, cx);
                    window.focus(&this.focus, cx);
                });
            },
            move |window, cx| {
                let _ = dismisser.update(cx, |this, cx| this.close_dropdown(window, cx));
            },
        )
    }
}

#[cfg(test)]
impl SettingsWindow {
    /// Why rows that edit Herdr's shared file are disabled, if they are. The
    /// pages read [`PageState::shared_gate`]; a test holding this window
    /// reads the same rule.
    pub(in crate::settings_window) fn shared_gate(&self) -> Option<String> {
        super::page_state::shared_gate(self.shared.as_ref())
    }
}

/// A callback for a control that reports a value rather than a click.
fn on_value<T: 'static>(
    cx: &Context<SettingsWindow>,
    act: impl Fn(&mut SettingsWindow, T, &mut Window, &mut Context<SettingsWindow>) + 'static,
) -> impl Fn(T, &mut Window, &mut App) + 'static {
    let view = cx.entity().downgrade();
    move |value, window, cx| {
        let _ = view.update(cx, |this, cx| act(this, value, window, cx));
    }
}

/// The names of `positions`, in order, for an override dropdown.
fn position_names<T>(positions: &[(T, &'static str)]) -> Vec<String> {
    positions
        .iter()
        .map(|(_, id)| i18n::message(id, &[]))
        .collect()
}

/// The name of `position` among `positions`, or nothing when it is missing.
fn position_name<T: PartialEq>(positions: &[(T, &'static str)], position: &T) -> String {
    position_message(positions, position).map_or_else(String::new, |id| i18n::message(id, &[]))
}

/// "On" or "Off" for a boolean override dropdown.
fn on_off(value: bool) -> String {
    i18n::message(spaceport_settings::fields::on_off_message(value), &[])
}

/// The choices of a boolean override dropdown, on first.
fn on_off_choices() -> [String; 2] {
    spaceport_settings::fields::on_off_messages().map(|id| i18n::message(id, &[]))
}
