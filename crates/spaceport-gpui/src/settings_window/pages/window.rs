//! Theme > Window and Terminal padding: what shows behind the main window,
//! and the space between each terminal and the edges of its tab group.
// i18n: migrated
use super::{PageState, SettingsWindow, on_value, padding_ids, ui};
use crate::{
    config::{Axis, Side, preferences::Preference},
    i18n::fl,
    settings_window::inputs::NumberField,
};
use gpui::{prelude::*, *};

/// The four padded sides, in the order the page lists them.
pub(in crate::settings_window) const PADDING_SIDES: [(Axis, Side); 4] = [
    (Axis::X, Side::Start),
    (Axis::X, Side::End),
    (Axis::Y, Side::Start),
    (Axis::Y, Side::End),
];

impl SettingsWindow {
    pub(in crate::settings_window) fn render_window_rows(
        &self,
        page: &PageState<'_>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = page.theme;
        div()
            .flex()
            .flex_col()
            .child(
                ui::section(
                    theme,
                    &fl!("settings-theme-section-window"),
                    Some(ui::Source::Spaceport),
                )
                .debug_selector(|| "settings-window-section".into()),
            )
            .child(self.render_backdrop_controls(cx))
            .child(
                ui::section(
                    theme,
                    &fl!("settings-theme-section-padding"),
                    Some(ui::Source::Spaceport),
                )
                .debug_selector(|| "settings-terminal-padding".into()),
            )
            .children(PADDING_SIDES.map(|(axis, side)| self.padding_row(page, axis, side, cx)))
    }

    fn padding_row(
        &self,
        page: &PageState<'_>,
        axis: Axis,
        side: Side,
        cx: &mut Context<Self>,
    ) -> ui::Row {
        let field = NumberField::TerminalPadding(axis, side);
        let (entry, _) = padding_ids(axis, side);
        let value = self.number_value(field, page.config.terminal_padding.axis(axis).side(side));
        let view = cx.entity().downgrade();
        ui::row(page.theme, entry).control(ui::number(
            page.theme,
            ui::Number {
                id: field.selector(),
                value,
                range: field.range(),
                unit: ui::Unit::Pixels,
                look: ui::Look::Slider,
                enabled: true,
            },
            self.number_edit_for(field),
            on_value(cx, move |this, change, _, cx| {
                this.commit_number(field, change, cx)
            }),
            move |window, cx| {
                let _ = view.update(cx, |this, cx| {
                    this.begin_number_edit(field, value, window, cx)
                });
            },
        ))
    }

    /// Saves one side; the other side of its axis is written along with it,
    /// since the file keeps them together.
    pub(in crate::settings_window) fn set_terminal_padding(
        &mut self,
        axis: Axis,
        side: Side,
        value: f32,
        cx: &mut Context<Self>,
    ) {
        let sides = self.config.terminal_padding.axis(axis);
        let moved = sides.stepped(side, value - sides.side(side));
        self.save_preference(Preference::TerminalPadding(axis, moved), cx);
    }
}
