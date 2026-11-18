// ABOUTME: Routes Settings-only Tab navigation without taking composition keys.
// ABOUTME: Keeps enabled control focus inside the independent Settings window.
use super::SettingsWindow;
use gpui::{Context, KeyDownEvent, Modifiers, Window, point};

impl SettingsWindow {
    pub(super) fn settings_tab(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let modifiers = event.keystroke.modifiers;
        if event.keystroke.key != "tab"
            || (modifiers != Modifiers::default() && modifiers != Modifiers::shift())
        {
            return false;
        }
        if self.settings_input_composing(window, cx) {
            return true;
        }
        let number_focused = self
            .number_edit
            .as_ref()
            .is_some_and(|edit| edit.input.read(cx).focus.is_focused(window));
        let hex_focused = self
            .hex_edit
            .as_ref()
            .is_some_and(|edit| edit.input.read(cx).focus.is_focused(window));
        if (number_focused && !self.finish_number_edit(true, cx))
            || (hex_focused && !self.finish_hex_edit(true, cx))
        {
            cx.stop_propagation();
            window.prevent_default();
            return true;
        }
        if self.dropdown.is_some() {
            self.close_dropdown(window, cx);
        }
        let modal = self.font_picker_shown();
        let mut first = None;
        // The rendered tab order is finite. Stop after at most one cycle.
        loop {
            if modifiers.shift {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            let focused = window.focused(cx);
            if focused.is_none() || focused == first {
                break;
            }
            if !modal
                || window.context_stack().iter().any(|context| {
                    context.contains("SettingsFontPicker") // i18n: data
                })
            {
                break;
            }
            if first.is_none() {
                first = focused;
            }
        }
        self.reveal_settings_focus(window, cx);
        cx.stop_propagation();
        window.prevent_default();
        true
    }
    fn reveal_settings_focus(&self, window: &Window, cx: &mut Context<Self>) {
        let contexts = window.context_stack();
        for (key, scroll) in [
            ("settings-row", &self.body_scroll),
            ("settings-section", &self.navigation_scroll),
        ] {
            let Some(id) = contexts.iter().rev().find_map(|context| context.get(key)) else {
                continue;
            };
            let Some(row) = cx
                .try_global::<super::search::FocusBounds>()
                .and_then(|bounds| bounds.0.get(id))
                .copied()
                .or_else(|| {
                    cx.try_global::<super::search::RowBounds>()
                        .and_then(|bounds| bounds.0.get(id))
                        .copied()
                })
            else {
                continue;
            };
            let bounds = scroll.bounds();
            let delta = if row.top() < bounds.top() || row.size.height > bounds.size.height {
                row.top() - bounds.top()
            } else {
                (row.bottom() - bounds.bottom()).max(gpui::px(0.))
            };
            let offset = scroll.offset();
            let y = (offset.y - delta)
                .max(-scroll.max_offset().y)
                .min(gpui::px(0.));
            scroll.set_offset(point(offset.x, y));
            cx.notify();
        }
    }
}
