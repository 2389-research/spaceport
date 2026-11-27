//! Status bar: a switch per item, and how compactly a shown item draws.
//! Usage and network quality keep their existing visibility keys,
//! which also stop their sampling.
// i18n: migrated
use super::{PageState, SettingsWindow, catalog, on_value, ui};
use crate::{
    config::{
        Config,
        preferences::Preference,
        status_bar::{AwakeScope, Button, Detail, Edit},
    },
    i18n::{self, fl},
};
use gpui::{prelude::*, *};

/// Each choice of a segmented control, with the message naming it.
const DETAILS: [(Detail, &str); 2] = [
    (Detail::Detailed, "settings-status-bar-detailed"),
    (Detail::Compact, "settings-status-bar-compact"),
];
const SCOPES: [(AwakeScope, &str); 2] = [
    (AwakeScope::Display, "settings-status-bar-awake-display"),
    (AwakeScope::System, "settings-status-bar-awake-system"),
];
const STYLES: [(Button, &str); 2] = [
    (Button::Label, "settings-status-bar-with-label"),
    (Button::Icon, "settings-status-bar-icon-only"),
];

/// The choices of a segmented control in the current locale.
fn choices<T: Copy>(choices: &[(T, &str)]) -> Vec<(T, String)> {
    choices
        .iter()
        .map(|&(value, message)| (value, i18n::message(message, &[])))
        .collect()
}

/// A status bar button row: its catalog entry, control ids, current value
/// and edit.
struct ButtonRow {
    entry: &'static catalog::Entry,
    toggle: &'static str,
    style: &'static str,
    current: Button,
    edit: fn(Button) -> Edit,
}

impl SettingsWindow {
    pub(in crate::settings_window) fn render_status_bar(
        &self,
        page: &PageState<'_>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = page.theme;
        let items = page.config.status_bar;
        let usage = page.config.usage.show;
        let network_quality = page.config.show_network_quality;
        let keep_awake = items.keep_awake;
        let buttons = [
            ButtonRow {
                entry: &catalog::STATUS_THEME,
                toggle: "status-theme-toggle",
                style: "status-theme-style",
                current: items.theme,
                edit: Edit::Theme,
            },
            ButtonRow {
                entry: &catalog::STATUS_SHORTCUTS,
                toggle: "status-shortcuts-toggle",
                style: "status-shortcuts-style",
                current: items.shortcuts,
                edit: Edit::Shortcuts,
            },
            ButtonRow {
                entry: &catalog::STATUS_REPORT_ISSUE,
                toggle: "status-report-issue-toggle",
                style: "status-report-issue-style",
                current: items.report_issue,
                edit: Edit::ReportIssue,
            },
        ];
        div()
            .flex()
            .flex_col()
            .child(ui::section(
                theme,
                &fl!("settings-status-bar-section-items"),
                Some(ui::Source::Spaceport),
            ))
            .child(
                ui::row(theme, &catalog::STATUS_USAGE)
                    .control(self.detail_control(
                        page,
                        "status-usage-detail",
                        items.usage,
                        usage,
                        Edit::Usage,
                        cx,
                    ))
                    .switch(
                        ui::toggle(theme, "status-usage-toggle", usage, true).on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.save_native(
                                    move || {
                                        Config::save_usage_visibility(!usage)
                                            .map_err(crate::Error::from)
                                    },
                                    cx,
                                );
                            }),
                        ),
                    ),
            )
            .child(
                ui::row(theme, &catalog::STATUS_NETWORK_QUALITY).switch(self.preference_toggle(
                    page.theme,
                    "status-network-quality-toggle",
                    network_quality,
                    Preference::ShowNetworkQuality(!network_quality),
                    cx,
                )),
            )
            .child(
                ui::row(theme, &catalog::STATUS_KEEP_AWAKE)
                    .control(ui::segmented(
                        theme,
                        "status-keep-awake-scope",
                        &choices(&SCOPES),
                        Some(items.keep_awake_scope),
                        true,
                        on_value(cx, |this, scope, _, cx| {
                            this.save_preference(
                                Preference::StatusBar(Edit::KeepAwakeScope(scope)),
                                cx,
                            )
                        }),
                    ))
                    .switch(self.preference_toggle(
                        page.theme,
                        "status-keep-awake-toggle",
                        keep_awake,
                        Preference::StatusBar(Edit::KeepAwake(!keep_awake)),
                        cx,
                    )),
            )
            .children(buttons.map(|button| self.button_row(page, button, cx)))
            .child(
                self.control_note(fl!("settings-status-bar-note"))
                    .mt(px(10.))
                    .text_size(px(11.)),
            )
    }

    /// Detailed or Compact for an item; inert while the item is hidden.
    fn detail_control(
        &self,
        page: &PageState<'_>,
        id: &'static str,
        current: Detail,
        shown: bool,
        edit: fn(Detail) -> Edit,
        cx: &mut Context<Self>,
    ) -> Div {
        ui::segmented(
            page.theme,
            id,
            &choices(&DETAILS),
            shown.then_some(current),
            shown,
            on_value(cx, move |this, detail, _, cx| {
                this.save_preference(Preference::StatusBar(edit(detail)), cx)
            }),
        )
    }

    /// A switch that hides the button, then whether it keeps its name.
    fn button_row(
        &self,
        page: &PageState<'_>,
        button: ButtonRow,
        cx: &mut Context<Self>,
    ) -> ui::Row {
        let ButtonRow {
            entry,
            toggle,
            style,
            current,
            edit,
        } = button;
        let shown = current.shown();
        let toggled = if shown { Button::Hidden } else { Button::Label };
        ui::row(page.theme, entry)
            .control(ui::segmented(
                page.theme,
                style,
                &choices(&STYLES),
                shown.then_some(current),
                shown,
                on_value(cx, move |this, choice, _, cx| {
                    this.save_preference(Preference::StatusBar(edit(choice)), cx)
                }),
            ))
            .switch(self.preference_toggle(
                page.theme,
                toggle,
                shown,
                Preference::StatusBar(edit(toggled)),
                cx,
            ))
    }
}

#[cfg(test)]
mod tests;
