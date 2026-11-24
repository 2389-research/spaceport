// ABOUTME: Implements Settings inputs interactions.
// ABOUTME: Connects native controls to Settings state and persistence.
//! Transient input state the pages share: the open dropdown, the number field
//! being typed into, and the root key routing for both.
// i18n: migrated
use super::{SettingsWindow, ui};
use crate::{config::preferences::Preference, search_input::SearchInput};
use gpui::{prelude::*, *};
pub(in crate::settings_window) use spaceport_settings::fields::{DropdownField, NumberField};
pub(in crate::settings_window) use spaceport_settings_window::inputs::NumberPreview;
use spaceport_settings_window::inputs::{self, Choice, picked};

/// The number field being typed into: the field, its text and whether what it
/// holds does not parse, with the blur that commits it.
pub(in crate::settings_window) struct NumberEdit {
    pub field: NumberField,
    pub input: Entity<SearchInput>,
    pub invalid: bool,
    pub _blur: Subscription,
}

impl SettingsWindow {
    pub(super) fn open_dropdown(
        &mut self,
        field: DropdownField,
        options: Vec<SharedString>,
        selected: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.persistence.quitting() {
            return;
        }
        self.dropdown = Some(ui::Dropdown::open(field, options, selected, window, cx));
        cx.notify();
    }

    /// Closes the list and hands focus back to the window unless the user
    /// already clicked into something else.
    pub(super) fn close_dropdown(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(open) = self.dropdown.take()
            && open.focus().is_focused(window)
        {
            window.focus(&self.focus, cx);
        }
        cx.notify();
    }

    /// Index 0 of an override dropdown follows Herdr; the rest are local choices.
    pub(super) fn pick_dropdown(
        &mut self,
        field: DropdownField,
        index: usize,
        cx: &mut Context<Self>,
    ) {
        self.dropdown = None;
        if self.persistence.quitting() || self.persistence.closing() {
            cx.notify();
            return;
        }
        let languages = super::pages::language_choices(&self.config.language);
        match picked(field, index, &languages) {
            Some(Choice::Layout(mode)) => self.accept_layout_choice(mode, cx),
            Some(Choice::Preference(edit)) => self.save_preference(edit, cx),
            None => {}
        }
        cx.notify();
    }

    pub(super) fn begin_number_edit(
        &mut self,
        field: NumberField,
        value: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let NumberField::FontSize(face) = field {
            self.controls.set_active_face(face);
        }
        if self.persistence.quitting() {
            return;
        }
        if !self.finish_number_edit(true, cx) {
            self.finish_number_edit(false, cx);
        }
        let input = cx.new(SearchInput::new);
        input.update(cx, |input, cx| {
            input.set_text_selected(&format!("{}", value as i64), cx);
            // The number's field already draws the frame.
            input.set_frameless(cx);
            input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
        });
        let focus = input.read(cx).focus.clone();
        let blur = cx.on_blur(&focus, window, |this, _, cx| {
            if !this.finish_number_edit(true, cx) {
                this.finish_number_edit(false, cx);
            }
        });
        self.number_edit = Some(NumberEdit {
            field,
            input,
            invalid: false,
            _blur: blur,
        });
        window.focus(&focus, cx);
        cx.notify();
    }

    /// Commits or abandons the typed number. False means the text is not yet
    /// acceptable (invalid or mid-composition) and the field stays open.
    pub(super) fn finish_number_edit(&mut self, save: bool, cx: &mut Context<Self>) -> bool {
        let Some(edit) = &mut self.number_edit else {
            return true;
        };
        let value = if save {
            if edit.input.read(cx).is_composing() {
                return false;
            }
            let range = edit.field.range();
            let Some(value) = ui::parse_number(edit.input.read(cx).text(), &range) else {
                edit.invalid = true;
                cx.notify();
                return false;
            };
            Some(value)
        } else {
            None
        };
        let field = edit.field;
        self.number_edit = None;
        if let Some(value) = value {
            self.commit_number(field, ui::NumberChange::Commit(value), cx);
        }
        cx.notify();
        true
    }

    /// Where each number field's changes go; previews only matter to sliders.
    pub(super) fn commit_number(
        &mut self,
        field: NumberField,
        change: ui::NumberChange,
        cx: &mut Context<Self>,
    ) {
        if let NumberField::Spacing(spacing) = field {
            let (ui::NumberChange::Preview(value) | ui::NumberChange::Commit(value)) = change
            else {
                return;
            };
            if field.range().contains(&value) {
                let commit = matches!(change, ui::NumberChange::Commit(_));
                self.set_spacing(spacing, Some(value), commit, cx);
            }
            return;
        }
        let value = match change {
            ui::NumberChange::Preview(value) => {
                // Shown at once; written only on release.
                let preview = NumberPreview::dragged(field, value);
                if preview.is_some() && self.number_preview != preview {
                    self.number_preview = preview;
                    cx.notify();
                }
                return;
            }
            ui::NumberChange::Commit(value) => value,
            ui::NumberChange::Abandon => {
                self.drop_number_preview(cx);
                return;
            }
        };
        let valid = !self.persistence.quitting() && field.range().contains(&value);
        // A busy window refuses the write, so only an idle one starts it.
        let started = valid && !self.busy();
        if valid {
            match field {
                NumberField::ChatMaxWidth => {
                    self.save_preference(Preference::ChatMaxWidth(value as u16), cx);
                }
                NumberField::ToastDelay => {
                    self.save_preference(Preference::NotificationDelay(Some(value as u64)), cx);
                }
                NumberField::SoundVolume => {
                    self.save_preference(Preference::SoundVolume(value as u8), cx);
                }
                NumberField::TerminalPadding(axis, side) => {
                    self.set_terminal_padding(axis, side, value, cx);
                }
                NumberField::FontSize(face) => self.accept_control_size(face, value, cx),
                NumberField::Spacing(_) => {}
            }
        }
        if let Some(preview) = self.number_preview.filter(|preview| preview.field == field) {
            // The released value stays until the save's reload shows it.
            self.number_preview = preview.after_commit(value, started, self.persistence.saving());
            cx.notify();
        }
    }

    /// What `field` shows: a live drag's value, else `saved`.
    pub(super) fn number_value(&self, field: NumberField, saved: f32) -> f32 {
        inputs::number_value(self.number_preview, field, saved)
    }

    /// Ends a slider preview unwritten: its drag was abandoned, its page
    /// went away, or a load now shows what the file holds.
    pub(super) fn drop_number_preview(&mut self, cx: &mut Context<Self>) {
        if self.number_preview.take().is_some() {
            cx.notify();
        }
    }

    pub(super) fn number_edit_for(&self, field: NumberField) -> Option<&NumberEdit> {
        self.number_edit.as_ref().filter(|edit| edit.field == field)
    }

    /// Root key handler: an open dropdown owns Up/Down/Enter/Escape, and a
    /// hex or number field being typed into owns Enter and Escape.
    pub(super) fn settings_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();
        if let Some(open) = &mut self.dropdown {
            if !matches!(key, "up" | "down" | "enter" | "escape") {
                return;
            }
            cx.stop_propagation();
            let field = open.field;
            match open.key(event) {
                Some(usize::MAX) => self.close_dropdown(window, cx),
                Some(index) => {
                    self.pick_dropdown(field, index, cx);
                    window.focus(&self.focus, cx);
                }
                None => cx.notify(),
            }
            return;
        }
        if !matches!(key, "enter" | "escape") {
            return;
        }
        // An invalid hex field stays open on blur, so both can be open: the
        // focused one takes the key, the hex field when neither is focused.
        let number_focused = self
            .number_edit
            .as_ref()
            .is_some_and(|edit| edit.input.read(cx).focus.is_focused(window));
        if !number_focused && let Some(edit) = &self.hex_edit {
            cx.stop_propagation();
            if edit.input.read(cx).is_composing() {
                return;
            }
            window.prevent_default();
            if self.finish_hex_edit(key == "enter", cx) {
                window.focus(&self.focus, cx);
            }
            return;
        }
        if self.number_edit.is_none() {
            let mut handled = self.search_key(key, window, cx);
            // Escape ends a theme card's preview while the pointer rests on it.
            if key == "escape" && self.theme_hovered() {
                self.hover_theme(None, cx);
                handled = true;
            }
            if handled {
                cx.stop_propagation();
            }
            return;
        }
        let Some(edit) = &self.number_edit else {
            return;
        };
        cx.stop_propagation();
        if edit.input.read(cx).is_composing() {
            return;
        }
        window.prevent_default();
        if self.finish_number_edit(key == "enter", cx) {
            window.focus(&self.focus, cx);
        }
    }
}
