// ABOUTME: Renders sidebar presentation preferences and their saved edits.
// ABOUTME: Shows Core-backed controls only when the running app opted into Core.
//! Sidebar: layout preset, selection marks, spacing and host colours. Every
//! change reaches the open windows at once; see `settings_window::sidebar`.
// i18n: migrated
use super::{SettingsWindow, catalog, on_value, ui};
use crate::{
    config::{
        AgentsPlacement, Config, DeviceFooter, LayoutMode, SelectMode, SidebarOverrides,
        preferences::Preference,
    },
    i18n::{self, fl},
    settings_window::{
        inputs::{DropdownField, NumberField},
        sidebar::SidebarIntent,
    },
    sidebar::SidebarMetrics,
};
use gpui::{prelude::*, *};
use std::ops::RangeInclusive;

/// The four spacing numbers the page edits. The old sidebar gap became the
/// terminal's left padding, on the Theme page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::settings_window) enum Spacing {
    Indent,
    RowPadding,
    RowGap,
    HostGap,
}

impl Spacing {
    pub(in crate::settings_window) const ALL: [Self; 4] =
        [Self::Indent, Self::RowPadding, Self::RowGap, Self::HostGap];

    /// The catalog entry of the row.
    pub(in crate::settings_window) const fn entry(self) -> &'static catalog::Entry {
        match self {
            Self::Indent => &catalog::SIDEBAR_INDENT,
            Self::RowPadding => &catalog::SIDEBAR_ROW_PADDING,
            Self::RowGap => &catalog::SIDEBAR_ROW_GAP,
            Self::HostGap => &catalog::SIDEBAR_HOST_GAP,
        }
    }

    pub(in crate::settings_window) fn control(self) -> &'static str {
        match self {
            Self::Indent => "sidebar-indent-control",
            Self::RowPadding => "sidebar-row-padding-control",
            Self::RowGap => "sidebar-row-gap-control",
            Self::HostGap => "sidebar-host-gap-control",
        }
    }

    fn reset(self) -> &'static str {
        match self {
            Self::Indent => "sidebar-indent-reset",
            Self::RowPadding => "sidebar-row-padding-reset",
            Self::RowGap => "sidebar-row-gap-reset",
            Self::HostGap => "sidebar-host-gap-reset",
        }
    }

    /// The `[sidebar]` key the parser bands.
    fn band(self) -> &'static str {
        match self {
            Self::Indent => "indent",
            Self::RowPadding => "row_padding",
            Self::RowGap => "gap",
            Self::HostGap => "host_gap",
        }
    }

    pub(in crate::settings_window) fn range(self) -> RangeInclusive<f32> {
        let key = self.band();
        let max = SidebarOverrides::BANDS
            .iter()
            .find_map(|(name, max)| (*name == key).then_some(*max))
            .unwrap_or(0.);
        0.0..=max
    }

    fn slot(self, overrides: &mut SidebarOverrides) -> &mut Option<f32> {
        match self {
            Self::Indent => &mut overrides.indent,
            Self::RowPadding => &mut overrides.row_padding,
            Self::RowGap => &mut overrides.gap,
            Self::HostGap => &mut overrides.host_gap,
        }
    }

    /// The configured override, or none when the preset applies.
    fn overridden(self, config: &Config) -> Option<f32> {
        let mut overrides = config.sidebar_style.overrides;
        *self.slot(&mut overrides)
    }

    /// What the sidebar uses now: the override, else the preset's value.
    fn value(self, config: &Config) -> f32 {
        let preset = SidebarMetrics::for_mode(config.layout.mode);
        self.overridden(config).unwrap_or(match self {
            // The card layouts step worktrees by their own padding.
            Self::Indent
                if matches!(config.layout.mode, LayoutMode::Superset | LayoutMode::Orca) =>
            {
                preset.card_indent
            }
            Self::Indent => preset.child_indent,
            Self::RowPadding => preset.row_padding,
            Self::RowGap => preset.gap,
            Self::HostGap => preset.host_gap,
        })
    }
}

impl SettingsWindow {
    /// Previews or commits one spacing number; `None` removes an override.
    pub(in crate::settings_window) fn set_spacing(
        &mut self,
        spacing: Spacing,
        value: Option<f32>,
        commit: bool,
        cx: &mut Context<Self>,
    ) {
        let mut intent = SidebarIntent::of(&self.config);
        *spacing.slot(&mut intent.style.overrides) = value;
        if commit {
            self.commit_sidebar(intent, cx);
        } else {
            self.preview_sidebar(intent, cx);
        }
    }

    pub(in crate::settings_window) fn render_sidebar(&self, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let ready = !self.quitting && self.closing.is_none();
        let mut page = div()
            .relative()
            .flex()
            .flex_col()
            .child(ui::section(
                theme,
                &fl!("settings-sidebar-section-layout"),
                None,
            ))
            .child(
                ui::row(theme, &catalog::SIDEBAR_LAYOUT)
                    .tag(ui::Source::Sheprd)
                    .control(self.layout_dropdown(ready, cx)),
            )
            .child(
                ui::row(theme, &catalog::SIDEBAR_ACTIVITY_BAR).switch(
                    ui::toggle(
                        theme,
                        "sidebar-activity-bar-toggle",
                        self.config.sidebar_style.activity_bar,
                        ready,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        let mut intent = SidebarIntent::of(&this.config);
                        intent.style.activity_bar = !intent.style.activity_bar;
                        this.commit_sidebar(intent, cx);
                    })),
                ),
            )
            .children(
                self.available(catalog::SIDEBAR_AUTOMATIONS.availability, cx)
                    .then(|| {
                        ui::row(theme, &catalog::SIDEBAR_AUTOMATIONS).switch(
                            ui::toggle(
                                theme,
                                "sidebar-automations-toggle",
                                self.config.sidebar_style.automations,
                                ready,
                            )
                            .on_click(cx.listener(|this, _, _, cx| {
                                let mut intent = SidebarIntent::of(&this.config);
                                intent.style.automations = !intent.style.automations;
                                this.commit_sidebar(intent, cx);
                            })),
                        )
                    }),
            )
            .child(
                ui::row(theme, &catalog::SIDEBAR_SELECT).control(ui::segmented(
                    theme,
                    "sidebar-select-control",
                    &[
                        (SelectMode::Row, fl!("settings-sidebar-select-row")),
                        (SelectMode::Group, fl!("settings-sidebar-select-group")),
                        (
                            SelectMode::GroupDim,
                            fl!("settings-sidebar-select-group-dim"),
                        ),
                    ],
                    Some(self.config.sidebar_style.select),
                    ready,
                    on_value(cx, |this, select, _, cx| {
                        let mut intent = SidebarIntent::of(&this.config);
                        intent.style.select = select;
                        this.commit_sidebar(intent, cx);
                    }),
                )),
            )
            .child(
                ui::row(theme, &catalog::SIDEBAR_DEVICE_FOOTER).control(ui::segmented(
                    theme,
                    "sidebar-device-footer-control",
                    &[
                        (DeviceFooter::Always, fl!("settings-sidebar-footer-always")),
                        (DeviceFooter::Auto, fl!("settings-sidebar-footer-auto")),
                        (DeviceFooter::Never, fl!("settings-sidebar-footer-never")),
                    ],
                    Some(self.config.sidebar_style.device_footer),
                    ready,
                    on_value(cx, |this, footer, _, cx| {
                        let mut intent = SidebarIntent::of(&this.config);
                        intent.style.device_footer = footer;
                        this.commit_sidebar(intent, cx);
                    }),
                )),
            )
            .child(
                ui::row(theme, &catalog::SIDEBAR_SHOW_AGENTS).switch(
                    ui::toggle(
                        theme,
                        "sidebar-show-agents-toggle",
                        self.config.show_agents,
                        true,
                    )
                    .on_click(cx.listener(|this, _, _, cx| {
                        let show = !this.config.show_agents;
                        this.save_native(move || Config::save_show_agents(show), cx);
                    })),
                ),
            )
            .child(ui::row(theme, &catalog::SIDEBAR_SHOW_AGENTS_HEADER).switch(
                self.preference_toggle(
                    "sidebar-show-agents-header-toggle",
                    self.config.show_agents_header,
                    Preference::ShowAgentsHeader(!self.config.show_agents_header),
                    cx,
                ),
            ))
            .child(
                ui::row(theme, &catalog::SIDEBAR_AGENTS_PLACEMENT).control(ui::segmented(
                    theme,
                    "sidebar-agents-placement-control",
                    &[
                        (
                            AgentsPlacement::Panel,
                            fl!("settings-sidebar-placement-panel"),
                        ),
                        (
                            AgentsPlacement::Nested,
                            fl!("settings-sidebar-placement-nested"),
                        ),
                    ],
                    Some(self.config.agents_placement),
                    ready && self.config.show_agents,
                    on_value(cx, |this, placement, _, cx| {
                        this.save_preference(Preference::AgentsPlacement(placement), cx);
                    }),
                )),
            )
            .child(
                ui::row(theme, &catalog::SIDEBAR_SHOW_SEARCH).switch(self.preference_toggle(
                    "sidebar-show-search-toggle",
                    self.config.show_sidebar_search,
                    Preference::ShowSidebarSearch(!self.config.show_sidebar_search),
                    cx,
                )),
            )
            .child(ui::row(theme, &catalog::SIDEBAR_REPOSITORY_ICONS).switch(
                self.preference_toggle(
                    "sidebar-repository-icons-toggle",
                    self.config.show_repository_icons,
                    Preference::ShowRepositoryIcons(!self.config.show_repository_icons),
                    cx,
                ),
            ))
            .child(
                ui::row(theme, &catalog::SIDEBAR_DEVICE_ICONS).switch(self.preference_toggle(
                    "sidebar-device-icons-toggle",
                    self.config.show_device_icons,
                    Preference::ShowDeviceIcons(!self.config.show_device_icons),
                    cx,
                )),
            )
            .child(ui::section(
                theme,
                &fl!("settings-sidebar-section-spacing"),
                None,
            ));
        for spacing in Spacing::ALL {
            page = page.child(self.spacing_row(spacing, ready, cx));
        }
        page = page.child(
            // A heading search opens, so it takes its entry's id and title.
            ui::section(theme, &catalog::SIDEBAR_HOST_COLOURS.title(), None)
                .relative()
                .debug_selector(|| catalog::SIDEBAR_HOST_COLOURS.id.into())
                .child(super::super::search::track_bounds(
                    catalog::SIDEBAR_HOST_COLOURS.id.into(),
                )),
        );
        let rows = self.host_rows(cx);
        if rows.is_empty() {
            page = page.child(
                div()
                    .debug_selector(|| "sidebar-host-colours-empty".into())
                    .py(px(6.))
                    .text_size(px(12.))
                    .text_color(rgb(theme.muted))
                    .child(fl!("settings-sidebar-hosts-empty")),
            );
        }
        let mut tinted = false;
        for (index, (host, color, hint)) in rows.into_iter().enumerate() {
            let hint = if hint.is_empty() && !tinted {
                tinted = true;
                "settings-sidebar-host-tint"
            } else {
                hint
            };
            let hint = if hint.is_empty() {
                SharedString::default()
            } else {
                i18n::message(hint, &[]).into()
            };
            let hex = self
                .hex_edit
                .as_ref()
                .filter(|edit| edit.host == host)
                .map(|edit| (edit.input.clone(), edit.invalid));
            let (changed, editing) = (host.clone(), host.clone());
            page = page.child(ui::swatch_row(
                theme,
                &host,
                index,
                color,
                &self.config.terminal,
                hint,
                hex,
                on_value(cx, move |this, change, _, cx| {
                    let color = match change {
                        ui::HostColorChange::Set(color) => Some(color),
                        ui::HostColorChange::Clear => None,
                    };
                    this.set_host_color(&changed, color, cx);
                }),
                {
                    let view = cx.entity().downgrade();
                    move |window, cx| {
                        let _ =
                            view.update(cx, |this, cx| this.begin_hex_edit(&editing, window, cx));
                    }
                },
            ));
        }
        #[cfg(all(feature = "integration-test", target_os = "macos"))]
        let page = page.child(super::super::native::probe(
            super::super::native::SIDEBAR_PAGE,
        ));
        page
    }

    fn layout_dropdown(&self, ready: bool, cx: &mut Context<Self>) -> Stateful<Div> {
        const FIELD: DropdownField = DropdownField::Layout;
        let mode = self.config.layout.mode;
        let options: Vec<SharedString> = LayoutMode::ALL
            .iter()
            .map(|mode| SharedString::from(mode.label()))
            .collect();
        let selected = LayoutMode::ALL
            .iter()
            .position(|candidate| *candidate == mode)
            .unwrap_or(0);
        let view = cx.entity().downgrade();
        let (opener, picker, dismisser) = (view.clone(), view.clone(), view);
        ui::dropdown(
            &self.theme,
            FIELD.selector(),
            mode.label(),
            false,
            ready,
            self.dropdown.as_ref().filter(|open| open.field == FIELD),
            move |window, cx| {
                let options = options.clone();
                let _ = opener.update(cx, |this, cx| {
                    this.open_dropdown(FIELD, options, selected, window, cx)
                });
            },
            move |index, window, cx| {
                let _ = picker.update(cx, |this, cx| {
                    this.pick_dropdown(FIELD, index, cx);
                    window.focus(&this.focus, cx);
                });
            },
            move |window, cx| {
                let _ = dismisser.update(cx, |this, cx| this.close_dropdown(window, cx));
            },
        )
    }

    fn spacing_row(&self, spacing: Spacing, ready: bool, cx: &mut Context<Self>) -> ui::Row {
        let field = NumberField::Spacing(spacing);
        let value = spacing.value(&self.config);
        let view = cx.entity().downgrade();
        ui::row(&self.theme, spacing.entry())
            .control(ui::number(
                &self.theme,
                ui::Number {
                    id: field.selector(),
                    value,
                    range: field.range(),
                    unit: ui::Unit::Pixels,
                    look: ui::Look::Slider,
                    enabled: ready,
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
            .control(self.spacing_reset(spacing, ready, cx))
    }

    /// The reset glyph, shown only while an override is set; a fixed slot
    /// keeps the numbers aligned either way.
    fn spacing_reset(&self, spacing: Spacing, ready: bool, cx: &mut Context<Self>) -> Div {
        let slot = div().w(px(16.)).flex_none();
        if spacing.overridden(&self.config).is_none() {
            return slot;
        }
        let id = spacing.reset();
        slot.child(
            div()
                .id(id)
                .debug_selector(move || id.into())
                .text_color(rgb(self.theme.muted))
                .when(ready, |reset| {
                    reset.cursor_pointer().on_click(
                        cx.listener(move |this, _, _, cx| {
                            this.set_spacing(spacing, None, true, cx)
                        }),
                    )
                })
                .child("\u{27f2}"),
        )
    }
}
