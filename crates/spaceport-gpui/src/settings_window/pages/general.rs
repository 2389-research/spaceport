// ABOUTME: Renders general preferences and the restart-only experimental Core opt-in.
// ABOUTME: Uses the shared preference writer for saved settings and feature toggles.
//! General: interface toggles, the optional `[features]` flags, clipboard
//! feedback, the browser skill and launch at login (Windows).
// i18n: migrated
use super::{
    CLIPBOARD_POSITIONS, PageState, SettingsWindow, catalog, on_off, on_off_choices, on_value,
    position_name, position_names, ui,
};
use crate::settings_window::inputs::{DropdownField, NumberField};
use crate::{
    agent_skill::{AgentSkill, Choice},
    config::{
        ChatLauncher, ChatLayoutMode, ClipboardToast, Feature, Label as _, preferences::Preference,
    },
    herdr_settings::Edit,
    i18n::fl,
};
use gpui::{prelude::*, *};

impl SettingsWindow {
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 218 lines")]
    pub(in crate::settings_window) fn render_general(
        &self,
        page: &PageState<'_>,
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = page.theme;
        let ready = self.controls_shared_ready();
        let copy = page.shared.is_none_or(|shared| shared.copy_on_select);
        let base = page.shared.map_or_else(
            || {
                let default = ClipboardToast::default();
                (default.enabled, default.position)
            },
            |shared| (shared.clipboard.enabled, shared.clipboard.position),
        );
        let overrides = page.config.clipboard_overrides;
        let position = overrides
            .position()
            .and_then(|p| CLIPBOARD_POSITIONS.iter().position(|(c, _)| *c == p));
        let position_names = position_names(&CLIPBOARD_POSITIONS);
        let base_position = position_name(&CLIPBOARD_POSITIONS, &base.1);
        let installed = AgentSkill::choice(cx) == Some(Choice::Installed);
        let (skill_id, skill_label, choice) = if installed {
            (
                "general-browser-skill-remove",
                fl!("settings-general-browser-skill-remove"),
                Choice::Declined,
            )
        } else {
            (
                "general-browser-skill-install",
                fl!("settings-general-browser-skill-install"),
                Choice::Installed,
            )
        };
        div()
            .flex()
            .flex_col()
            .child(ui::section(
                theme,
                &fl!("settings-general-section-interface"),
                None,
            ))
            .child(self.language_row(page, cx))
            .child(
                ui::row(theme, &catalog::GENERAL_PORTS).switch(self.preference_toggle(
                    page.theme,
                    "general-ports-toggle",
                    page.config.show_listening_ports,
                    Preference::ShowListeningPorts(!page.config.show_listening_ports),
                    cx,
                )),
            )
            .child(
                ui::row(theme, &catalog::GENERAL_SMOOTH_SCROLL).switch(self.preference_toggle(
                    page.theme,
                    "general-smooth-scroll-toggle",
                    page.config.smooth_scroll,
                    Preference::SmoothScroll(!page.config.smooth_scroll),
                    cx,
                )),
            )
            .when(
                page.available(catalog::GENERAL_WINDOW_CONTROLS.availability),
                |shown| {
                    shown.child(ui::row(theme, &catalog::GENERAL_WINDOW_CONTROLS).switch(
                        self.preference_toggle(
                            page.theme,
                            "general-window-controls-toggle",
                            page.config.show_window_controls,
                            Preference::ShowWindowControls(!page.config.show_window_controls),
                            cx,
                        ),
                    ))
                },
            )
            .child(ui::section(
                theme,
                &fl!("settings-general-section-experimental"),
                None,
            ))
            .child(
                ui::row(theme, &catalog::GENERAL_HERDR_CORE).switch(self.preference_toggle(
                    page.theme,
                    "general-herdr-core-toggle",
                    page.config.herdr_core_enabled,
                    Preference::HerdrCoreEnabled(!page.config.herdr_core_enabled),
                    cx,
                )),
            )
            .children(
                crate::core_feature::restart_required(page.config.herdr_core_enabled, cx).then(
                    || {
                        div()
                            .debug_selector(|| "general-herdr-core-restart".into())
                            .text_size(px(11.))
                            .text_color(rgb(theme.subtext()))
                            .child(fl!("settings-general-herdr-core-restart"))
                    },
                ),
            )
            .child(ui::section(
                theme,
                &fl!("settings-general-section-features"),
                None,
            ))
            .children(self.feature_rows(page, cx))
            .children(
                page.available(catalog::GENERAL_CHAT_LAYOUT.availability)
                    .then(|| {
                        let layouts: Vec<_> = ChatLayoutMode::ALL
                            .iter()
                            .map(|&mode| (mode, mode.label()))
                            .collect();
                        ui::row(theme, &catalog::GENERAL_CHAT_LAYOUT).control(ui::segmented(
                            theme,
                            "general-chat-layout-control",
                            &layouts,
                            Some(page.config.chat.layout),
                            !self.persistence.quitting() && !self.persistence.closing(),
                            on_value(cx, |this, mode, _, cx| {
                                this.save_preference(Preference::ChatLayout(mode), cx)
                            }),
                        ))
                    }),
            )
            .children(
                page.available(catalog::GENERAL_CHAT_FULL_WIDTH.availability)
                    .then(|| self.chat_width_controls(page, cx)),
            )
            .children(
                page.available(catalog::GENERAL_CHAT_LAUNCHER.availability)
                    .then(|| {
                        let launchers: Vec<_> = ChatLauncher::ALL
                            .iter()
                            .map(|&launcher| (launcher, launcher.label()))
                            .collect();
                        ui::row(theme, &catalog::GENERAL_CHAT_LAUNCHER).control(ui::segmented(
                            theme,
                            "general-chat-launcher-control",
                            &launchers,
                            Some(page.config.chat.launcher),
                            !self.persistence.quitting() && !self.persistence.closing(),
                            on_value(cx, |this, launcher, _, cx| {
                                this.save_preference(Preference::ChatLauncher(launcher), cx)
                            }),
                        ))
                    }),
            )
            .children(
                page.available(catalog::GENERAL_CHAT_GLIDE.availability)
                    .then(|| {
                        ui::row(theme, &catalog::GENERAL_CHAT_GLIDE).switch(self.preference_toggle(
                            page.theme,
                            "general-chat-glide-toggle",
                            page.config.chat.glide,
                            Preference::ChatGlide(!page.config.chat.glide),
                            cx,
                        ))
                    }),
            )
            .children(self.render_chat_env_controls(cx))
            .child(ui::section(
                theme,
                &fl!("settings-general-section-clipboard"),
                None,
            ))
            .child(
                page.gate_shared(
                    ui::row(theme, &catalog::GENERAL_COPY_ON_SELECT).tag(ui::Source::Herdr),
                )
                .switch(
                    ui::toggle(theme, "general-copy-on-select-toggle", copy, ready).when(
                        ready,
                        |toggle| {
                            toggle.on_click(cx.listener(move |this, _, _, cx| {
                                this.save_shared(Edit::CopyOnSelect(!copy), cx)
                            }))
                        },
                    ),
                ),
            )
            .child(
                ui::row(theme, &catalog::GENERAL_COPIED_TOAST)
                    .tag(ui::Source::Spaceport)
                    .control(self.override_dropdown(
                        page.theme,
                        DropdownField::ClipboardEnabled,
                        &on_off(base.0),
                        &on_off_choices(),
                        overrides.enabled().map(|on| usize::from(!on)),
                        cx,
                    )),
            )
            .child(
                ui::row(theme, &catalog::GENERAL_CLIPBOARD_POSITION)
                    .tag(ui::Source::Spaceport)
                    .control(self.override_dropdown(
                        page.theme,
                        DropdownField::ClipboardPosition,
                        &base_position,
                        &position_names,
                        position,
                        cx,
                    )),
            )
            .child(ui::section(
                theme,
                &fl!("settings-general-section-browser-skill"),
                None,
            ))
            .child(
                ui::row(theme, &catalog::GENERAL_BROWSER_SKILL)
                    // Until it is installed the row reports that instead.
                    .when(!installed, |row| {
                        row.description(fl!("settings-general-browser-skill-missing"))
                    })
                    .control(
                        ui::button(theme, skill_id, skill_label, true).on_click(
                            cx.listener(move |this, _, _, cx| this.save_skill(choice, cx)),
                        ),
                    ),
            )
            .children(self.startup_rows(page, cx))
    }

    /// Windows' **Launch at login**; other platforms have no such setting yet.
    fn startup_rows(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> Vec<AnyElement> {
        #[cfg(windows)]
        return self.launch_at_login_rows(page, cx);
        #[cfg(not(windows))]
        {
            let _ = (page, cx);
            Vec::new()
        }
    }

    /// One row per `[features]` flag, generated from [`Feature::ALL`] so a
    /// flag added to the table shows up here on its own.
    fn feature_rows(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> Vec<ui::Row> {
        let features = page.config.features;
        Feature::ALL
            .iter()
            .map(|&feature| {
                let id = feature.setting_id();
                let on = features.enabled(feature);
                ui::row(page.theme, &catalog::feature(feature)).switch(self.preference_toggle(
                    page.theme,
                    format!("{id}-toggle"),
                    on,
                    Preference::Feature(feature, !on),
                    cx,
                ))
            })
            .collect()
    }

    pub(in crate::settings_window) fn save_skill(
        &mut self,
        choice: Choice,
        cx: &mut Context<Self>,
    ) {
        self.save_skill_with(
            choice,
            move || {
                let home = crate::config::home()?;
                match choice {
                    Choice::Installed => {
                        let text =
                            crate::agent_skill::text(crate::agent_skill::executable().as_deref());
                        crate::agent_skill::install(&home, &text)?;
                    }
                    Choice::Declined => {
                        crate::agent_skill::remove(&home)?;
                    }
                }
                Ok(())
            },
            self.loader(cx),
            cx,
        );
    }

    pub(in crate::settings_window) fn save_skill_with(
        &mut self,
        choice: Choice,
        operation: impl FnOnce() -> crate::Result<()> + Send + 'static,
        load: impl FnOnce() -> crate::Result<super::super::Loaded> + Send + 'static,
        cx: &mut Context<Self>,
    ) {
        self.save_with_completion(
            operation,
            load,
            false,
            move |cx| AgentSkill::choose(choice, cx),
            cx,
        );
    }

    fn chat_width_controls(&self, page: &PageState<'_>, cx: &mut Context<Self>) -> Div {
        let full = page.config.chat.max_width == 0;
        let field = NumberField::ChatMaxWidth;
        let value = self.number_value(field, f32::from(page.config.chat.max_width));
        let view = cx.entity().downgrade();
        div()
            .child(
                ui::row(page.theme, &catalog::GENERAL_CHAT_FULL_WIDTH).switch(
                    self.preference_toggle(
                        page.theme,
                        "general-chat-full-width-toggle",
                        full,
                        Preference::ChatMaxWidth(if full {
                            crate::config::chat::DEFAULT_MAX_WIDTH
                        } else {
                            0
                        }),
                        cx,
                    ),
                ),
            )
            .when(
                page.available(catalog::GENERAL_CHAT_WIDTH.availability),
                |controls| {
                    controls.child(ui::row(page.theme, &catalog::GENERAL_CHAT_WIDTH).control(
                        ui::number(
                            page.theme,
                            ui::Number {
                                id: field.selector(),
                                value,
                                range: field.range(),
                                unit: ui::Unit::Pixels,
                                look: ui::Look::Slider,
                                enabled: !self.persistence.quitting()
                                    && !self.persistence.closing(),
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
                        ),
                    ))
                },
            )
    }
}
