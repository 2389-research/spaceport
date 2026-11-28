// ABOUTME: Renders tab and pane preferences while hiding disabled Core controls.
// ABOUTME: Keeps saved terminal choices separate from the running Core opt-in.
//! Tabs & panes: tab layout and bar, closing confirmations, pane focus and
//! session restore.
// i18n: migrated
use super::{PageState, SettingsWindow, catalog, on_value, ui};
use crate::{
    config::{Label as _, TabLayoutMode, preferences::Preference},
    herdr_settings::{Edit, TabBarPosition},
    i18n::fl,
    settings_window::remote_history::HostState,
};
use gpui::{prelude::*, *};

impl SettingsWindow {
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 171 lines")]
    pub(in crate::settings_window) fn render_tabs(
        &self,
        page: &PageState<'_>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = page.theme;
        let ready = self.controls_shared_ready();
        let shared = page.shared;
        let position = shared.map(|shared| shared.tab_bar_position);
        let hide = shared.is_some_and(|shared| shared.hide_tab_bar_when_single_tab);
        let history = shared.is_some_and(|shared| shared.pane_history);
        let layouts: Vec<_> = TabLayoutMode::ALL
            .iter()
            .map(|&mode| (mode, mode.label()))
            .collect();
        let open = !self.persistence.quitting() && !self.persistence.closing();
        let mut rows = div()
            .flex()
            .flex_col()
            .child(ui::section(
                theme,
                &fl!("settings-tabs-section-layout"),
                None,
            ))
            .child(ui::row(theme, &catalog::TABS_LAYOUT).control(ui::segmented(
                theme,
                "tabs-layout-control",
                &layouts,
                Some(page.config.tab_layout),
                open,
                on_value(cx, |this, mode, _, cx| {
                    this.save_preference(Preference::TabLayout(mode), cx)
                }),
            )))
            .child(
                ui::row(theme, &catalog::TABS_AGENT_TITLE).switch(self.preference_toggle(
                    page.theme,
                    "tabs-agent-title-toggle",
                    page.config.tab_labels.agent_title,
                    Preference::TabAgentTitle(!page.config.tab_labels.agent_title),
                    cx,
                )),
            )
            .children(page.available(catalog::TABS_HERDR_2.availability).then(|| {
                ui::row(theme, &catalog::TABS_HERDR_2).switch(self.preference_toggle(
                    page.theme,
                    "tabs-herdr-2-toggle",
                    page.config.herdr_2_terminals,
                    Preference::Herdr2Terminals(!page.config.herdr_2_terminals),
                    cx,
                ))
            }))
            .child(ui::section(
                theme,
                &fl!("settings-tabs-section-tab-bar"),
                Some(ui::Source::Herdr),
            ))
            .child(
                page.gate_shared(ui::row(theme, &catalog::TABS_POSITION))
                    .control(ui::segmented(
                        theme,
                        "tabs-position-control",
                        &[
                            (TabBarPosition::Top, fl!("settings-tabs-position-top")),
                            (TabBarPosition::Bottom, fl!("settings-tabs-position-bottom")),
                        ],
                        position,
                        ready,
                        on_value(cx, |this, choice, _, cx| {
                            this.save_shared(Edit::TabBarPosition(choice), cx)
                        }),
                    )),
            )
            .child(
                page.gate_shared(ui::row(theme, &catalog::TABS_HIDE_SINGLE))
                    .switch(
                        ui::toggle(theme, "tabs-hide-single-toggle", hide, ready).when(
                            ready,
                            |toggle| {
                                toggle.on_click(cx.listener(move |this, _, _, cx| {
                                    this.save_shared(Edit::HideSingleTabBar(!hide), cx)
                                }))
                            },
                        ),
                    ),
            )
            .child(ui::section(
                theme,
                &fl!("settings-tabs-section-closing"),
                None,
            ))
            .child(
                ui::row(theme, &catalog::TABS_CONFIRM_TAB).switch(self.preference_toggle(
                    page.theme,
                    "tabs-confirm-tab-toggle",
                    page.config.confirm_close_tab,
                    Preference::ConfirmCloseTab(!page.config.confirm_close_tab),
                    cx,
                )),
            )
            .child(
                ui::row(theme, &catalog::TABS_CONFIRM_PANE).switch(self.preference_toggle(
                    page.theme,
                    "tabs-confirm-pane-toggle",
                    page.config.confirm_close_pane,
                    Preference::ConfirmClosePane(!page.config.confirm_close_pane),
                    cx,
                )),
            )
            .child(ui::section(
                theme,
                &fl!("settings-tabs-section-panes"),
                None,
            ))
            .child(ui::row(theme, &catalog::TABS_FOCUS_FOLLOWS_MOUSE).switch(
                self.preference_toggle(
                    page.theme,
                    "tabs-focus-follows-mouse-toggle",
                    page.config.focus_follows_mouse,
                    Preference::FocusFollowsMouse(!page.config.focus_follows_mouse),
                    cx,
                ),
            ))
            .child(ui::section(
                theme,
                &fl!("settings-tabs-section-session-restore"),
                None,
            ))
            .child(
                page.gate_shared(
                    ui::row(theme, &catalog::TABS_PANE_HISTORY).tag(ui::Source::Herdr),
                )
                .switch(
                    ui::toggle(theme, "tabs-pane-history-toggle", history, ready).when(
                        ready,
                        |toggle| {
                            toggle.on_click(cx.listener(move |this, _, _, cx| {
                                this.save_shared(Edit::PaneHistory(!history), cx)
                            }))
                        },
                    ),
                ),
            );
        for (index, host) in self.remote_history.hosts().iter().enumerate() {
            let (checked, settled) = match &host.state {
                HostState::Ready(history) => (history.enabled, true),
                HostState::Saving(history) => (history.enabled, false),
                HostState::Loading | HostState::Failed(_) => (false, false),
            };
            let enabled = open && settled;
            let target = host.target.clone();
            let note = match &host.state {
                HostState::Loading => fl!("settings-tabs-host-loading"),
                HostState::Saving(_) => fl!("settings-tabs-host-saving"),
                HostState::Failed(error) => {
                    fl!("settings-tabs-host-failed", error = error.to_string())
                }
                HostState::Ready(_) => fl!("settings-tabs-host-ready"),
            };
            let id = format!("tabs-pane-history-host-{index}");
            rows = rows.child(
                ui::data_row(theme, id.clone(), host.label.clone())
                    .description(note)
                    .switch(
                        ui::toggle(theme, format!("{id}-toggle"), checked, enabled).when(
                            enabled,
                            |toggle| {
                                toggle.on_click(cx.listener(move |this, _, _, cx| {
                                    this.save_remote_history(&target, !checked, cx);
                                }))
                            },
                        ),
                    ),
            );
        }
        rows
    }
}
