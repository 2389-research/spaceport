// ABOUTME: Loads settings and presents the running app's keyboard shortcut reference.
// ABOUTME: Keeps unavailable Core commands out of the visible shortcut catalog.
//! The Settings and Keybinds pages, and the GUI config load behind them. The
//! load is a cancellable background task: config parsing never runs on the UI
//! thread, and a superseded load cannot overwrite a newer one.

// i18n: migrated
use super::{Page, accent};
use crate::i18n::fl;
use crate::{
    SpaceportWindow,
    config::{Config, FontConfig, ThemeName},
    controls::{COMMANDS, Command, CommandInfo},
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::ClientShellCommand;
use spaceport_keymap::Reach;
use spaceport_settings::Appearance;
use std::borrow::Cow;

/// The semantic paste row, which no keymap command holds.
fn paste_label() -> String {
    fl!("menu-settings-paste")
}

impl SpaceportWindow {
    pub(crate) fn reload_notification_config(&mut self, cx: &mut Context<Self>) {
        self.sound.reload();
        use crate::config::NotificationDelivery::Off;
        let was_off = self.config.notifications.delivery() == Off;
        if let Some(shared) = &self.settings.shared {
            self.config.apply_shared_notifications(shared);
        }
        let now = std::time::Instant::now();
        for endpoint in &mut self.endpoints {
            // Turning delivery on, in-app or system, never replays the backlog.
            if was_off && self.config.notifications.delivery() != Off {
                endpoint.toasts.enabled_since = Some(now);
            }
        }
        self.tick_toasts(self.menu.page.is_some() || self.toasts_hidden, now, cx);
        cx.notify();
    }

    /// Reloads when the GUI overrides or the private view settings change.
    /// The latter still holds `[keys]`, clipboard toast, and `[ui.sidebar]`.
    /// Reads the theme again when a theme file changes in place.
    pub(crate) fn watch_gui_config(&mut self, cx: &mut Context<Self>) {
        let Ok(path) = Config::local_path() else {
            return;
        };
        let daemon = crate::config::view_settings_path(spaceport_client::local_var);
        let executor = cx.background_executor().clone();
        let mut theme = self.config.theme.clone();
        self.config_watch = Some(cx.spawn(async move |this, cx| {
            let mut watch = crate::config::watch::Watch::default();
            let mut theme_watch = crate::config::watch::ThemeWatch::default();
            let mut pending = None;
            loop {
                let (path, daemon) = (path.clone(), daemon.clone());
                let (sample, theme_sample, sampled) = executor
                    .spawn(async move {
                        use crate::config::watch::{fingerprint, fingerprint_all};
                        let theme_sample = fingerprint_all(Config::theme_files(&theme));
                        (
                            [fingerprint(&path), fingerprint(&daemon)],
                            theme_sample,
                            theme,
                        )
                    })
                    .await;
                theme = sampled;
                let updated = this.update(cx, |this, cx| {
                    if theme_watch.observe(&theme, theme_sample, &this.config.theme)
                        && this.config_load.is_none()
                        && this.reload_theme(cx)
                    {
                        theme_watch.accept(theme_sample);
                    }
                    theme.clone_from(&this.config.theme);
                    if let Some((sample, revision)) = pending
                        && this.config_load_revision != revision
                    {
                        watch.accept(sample);
                        pending = None;
                    }
                    if watch.observe(sample)
                        && this.config_load.is_none()
                        && this.settings.task.is_none()
                        && !matches!(this.menu.page, Some(Page::Themes))
                        && !this.theme_save_in_flight()
                    {
                        this.load_gui_config(cx);
                        this.load_shared_settings(cx);
                        pending = Some((sample, this.config_load_revision));
                    }
                });
                if updated.is_err() {
                    break;
                }
                executor.timer(std::time::Duration::from_millis(250)).await;
            }
        }));
    }

    pub(crate) fn open_keybinds(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open_menu(window, cx) {
            return;
        }
        self.menu.page = Some(Page::Keybinds);
        self.menu.keybinds_scroll.set_offset(Point::default());
        let search = cx.new(crate::search_input::SearchInput::new);
        search.update(cx, |input, cx| {
            input.set_placeholder_with(|| fl!("menu-settings-search"), cx);
            input.set_appearance(self.config.ui.clone(), self.theme.clone(), cx);
            window.focus(&input.focus, cx);
        });
        self.menu._keybinds_subscription = Some(cx.subscribe(
            &search,
            |this, _, _: &crate::search_input::Changed, cx| {
                this.menu.keybinds_scroll.set_offset(Point::default());
                cx.notify();
            },
        ));
        self.menu.keybinds_search = Some(search);
    }

    pub(crate) fn open_preferences(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.theme_save_in_flight() {
            return;
        }
        self.dismiss_menu(window, cx);
        crate::settings_window::open(cx.weak_entity(), cx);
    }

    pub(crate) fn reload_gui_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.theme_save_in_flight() {
            return;
        }
        self.load_gui_config(cx);
        self.dismiss_menu(window, cx);
    }

    pub(crate) fn load_gui_config(&mut self, cx: &mut Context<Self>) {
        // Enumerating installed families is slow, so it rides the same
        // background load as parsing rather than the UI thread.
        let light = crate::app::light_appearance(cx);
        let text_system = cx.text_system().clone();
        self.load_gui_config_with(
            move || {
                let mut config = Config::load()?;
                config.resolve_fonts(|| text_system.all_font_names());
                // Built beside the settings it came from: an invalid reload
                // keeps the previous working pair.
                let keymap = crate::config::keymap(&config)?;
                // Follow Herdr is resolved from the latest prepared snapshot on completion.
                let theme = if ThemeName::follows_herdr(&config.theme, light) {
                    Default::default()
                } else {
                    config.theme(light)?
                };
                Ok((config, keymap, theme))
            },
            cx,
        );
    }

    pub(crate) fn load_gui_config_with(
        &mut self,
        load: impl FnOnce() -> crate::Result<(Config, spaceport_keymap::Keymap, crate::config::Theme)>
        + Send
        + 'static,
        cx: &mut Context<Self>,
    ) {
        if self.config_load.is_some() {
            return;
        }
        let theme_revision = crate::settings_window::theme_load_revision(cx);
        let layout_revision = crate::settings_window::layout_load_revision(cx);
        let sidebar_revision = crate::settings_window::sidebar_load_revision(cx);
        let layouts = cx.default_global::<crate::window::LayoutSaves>().stamp();
        // Loaders resolve the theme for the appearance when they start.
        let light = crate::app::light_appearance(cx);
        let load = cx.background_executor().spawn(async move { load() });
        self.config_load = Some(cx.spawn(async move |this, cx| {
            let loaded = load.await;
            let _ = this.update(cx, |this, cx| {
                this.config_load = None;
                this.config_load_revision = this.config_load_revision.wrapping_add(1);
                // Apply a coherent pair only after both have loaded successfully.
                match loaded {
                    Ok((mut config, keymap, mut theme)) => {
                        // Before the Settings drafts: a draft being edited wins.
                        cx.default_global::<crate::window::LayoutSaves>()
                            .keep_newer(layouts, &mut config);
                        crate::settings_window::apply_loaded_layout(&mut config, layout_revision, cx);
                        crate::settings_window::apply_loaded_sidebar(&mut config, sidebar_revision, cx);
                        crate::settings_window::apply_loaded_theme(&mut config, &mut theme, theme_revision, cx);
                        if let Some(shared) = &this.settings.shared {
                            config.apply_shared_notifications(shared);
                        }
                        // A reload discards session zoom.
                        this.configured_terminal_size = config.terminal.size;
                        cx.set_global(crate::app::InitialAppearance {
                            error: None,
                            keymap: keymap.clone(),
                            shown: Appearance {
                                config: config.clone(),
                                theme: theme.clone(),
                            },
                        });
                        crate::caffeine::set_scope(config.status_bar.keep_awake_scope, cx);
                        crate::i18n::follow_config(&config.language, cx);
                        let menus_changed = crate::menus::changed(&this.config, &config);
                        if keymap != this.keymap {
                            crate::actions::rebind_keys(cx);
                        } else if menus_changed {
                            crate::menus::install(cx);
                        }
                        if menus_changed {
                            this.close_stale_application_menu(cx);
                        }
                        if this.config.notifications.delivery() == crate::config::NotificationDelivery::Off
                            && config.notifications.delivery() != crate::config::NotificationDelivery::Off
                        {
                            let cutoff = std::time::Instant::now();
                            for endpoint in &mut this.endpoints {
                                endpoint.toasts.enabled_since = Some(cutoff);
                            }
                        }
                        if config.status_bar.show != this.config.status_bar.show {
                            this.status_bar_visible = config.status_bar.show;
                        }
                        this.config = config.clone();
                        this.keymap = keymap;
                        if !ThemeName::follows_herdr(&this.config.theme, light) {
                            this.theme = theme;
                        }
                        this.apply_shared_theme(cx);
                        this.theme_light = light;
                        if light != crate::app::light_appearance(cx) {
                            this.apply_system_theme(cx);
                        }
                        crate::settings_window::apply_loaded_theme(&mut this.config, &mut this.theme, theme_revision, cx);
                        cx.set_global(crate::app::InitialAppearance {
                            error: None,
                            keymap: this.keymap.clone(),
                            shown: Appearance {
                                config: this.config.clone(),
                                theme: this.theme.clone(),
                            },
                        });
                        this.config = config;
                        this.sound.set_volume(this.config.sound.volume);
                        this.gui_config_diagnostic
                            .sync_own(crate::config::diagnostic(&this.config).as_deref());
                        crate::settings_window::apply_loaded_theme(&mut this.config, &mut this.theme, theme_revision, cx);
                        this.tick_toasts(
                            this.menu.page.is_some() || this.toasts_hidden,
                            std::time::Instant::now(),
                            cx,
                        );
                        crate::log_window::set_appearance(&this.config, &this.theme, cx);
                        this.wheel = Default::default();
                        this.last_queued_options = None;
                        this.notices.clear();
                    }
                    Err(error) => {
                        tracing::warn!(%error, "Could not load GUI config; keeping current settings");
                        let message = fl!("menu-settings-load-failed", error = error.to_string());
                        this.notices.show(crate::notice::Cause::Configuration, message);
                    }
                }
                cx.notify();
            });
        }));
    }
}

impl SpaceportWindow {
    pub(super) fn render_keybinds(&self, cx: &mut Context<Self>) -> Div {
        let font = &self.config.ui;
        let query = self
            .menu
            .keybinds_search
            .as_ref()
            .map(|search| search.read(cx).text().to_owned())
            .unwrap_or_default();
        let groups = self.keybind_groups();
        let total: usize = groups.iter().map(|(_, rows)| rows.len()).sum();
        let body = div()
            .id("keybinds-body")
            .debug_selector(|| "keybinds-body".into())
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.menu.keybinds_scroll)
            .px(px(16.))
            .py(px(8.))
            .children(self.render_shortcut_editor(cx));
        let (body, count) = self.keybind_body(body, groups, &query, cx);
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .child(self.keybind_header(cx))
            .child(self.keybind_search_area(count, total, font))
            .child(body)
            .child(Self::keybind_footer())
    }

    /// Every row the page lists, by section: the catalog's commands with the
    /// keys the keymap gives them, then the daemon's own commands.
    fn keybind_groups<'a>(&'a self) -> Vec<(String, Vec<KeybindRow<'a>>)> {
        let paste = paste_label();
        let mut groups = vec![
            (fl!("menu-settings-workspaces"), Vec::new()),
            (fl!("menu-settings-navigation"), Vec::new()),
            (
                fl!("menu-settings-application"),
                vec![KeybindRow {
                    description: paste.clone(),
                    info: None,
                    keys: vec![(Cow::Borrowed(crate::actions::EDIT_KEYS[2]), None)],
                }],
            ),
            (fl!("menu-settings-commands"), Vec::new()),
        ];
        for info in COMMANDS {
            if !self.core_enabled() && crate::core_feature::command_requires_core(info.command) {
                continue;
            }
            let mut keys: Vec<&'a str> = self.keymap().shortcuts(info.command).collect();
            if info.command == Command::Palette && self.config.palette.double_shift {
                keys.push("shift shift"); // i18n: data
            }
            let section = keybind_section(info.command);
            groups[section].1.push(KeybindRow {
                description: crate::i18n::command(info),
                info: Some(info),
                keys: keys
                    .into_iter()
                    .map(|key| (Cow::Borrowed(key), None))
                    .collect(),
            });
        }
        if let Some(snapshot) = self
            .live
            .snapshot
            .as_ref()
            .filter(|_| self.live.status.is_connected())
        {
            let commands: Vec<_> =
                spaceport_keymap::available_commands(&snapshot.commands).collect();
            let bindings = self.keymap().custom_bindings(commands.iter().copied());
            groups[3]
                .1
                .extend(commands.iter().zip(bindings).enumerate().map(
                    |(index, (command, bindings))| {
                        let description = command_name(command);
                        // A row's name keys its selectors and is all that tells
                        // rows apart, so one already used by a built-in action
                        // or an earlier command is numbered. Daemon command IDs
                        // are generated per boot and mean nothing to a reader.
                        let earlier = usize::from(
                            description == paste
                                || COMMANDS
                                    .iter()
                                    .any(|info| crate::i18n::command(info) == description),
                        ) + commands[..index]
                            .iter()
                            .filter(|other| command_name(other) == description)
                            .count();
                        let description = match earlier {
                            0 => description.to_owned(),
                            earlier => format!("{description} ({})", earlier + 1),
                        };
                        KeybindRow {
                            description,
                            info: None,
                            keys: bindings
                                .into_iter()
                                .map(|binding| {
                                    (Cow::Owned(binding.label), reach_note(binding.reach))
                                })
                                .collect(),
                        }
                    },
                ));
        }
        groups
    }

    /// The rows `query` keeps, under their sections, and how many matched.
    fn keybind_body(
        &self,
        body: Stateful<Div>,
        groups: Vec<(String, Vec<KeybindRow<'_>>)>,
        query: &str,
        cx: &mut Context<Self>,
    ) -> (Stateful<Div>, usize) {
        let mut body = body;
        let mut count = 0;
        for (section, rows) in groups {
            let rows: Vec<_> = rows
                .into_iter()
                .filter(|row| {
                    shortcut_matches(query, "", &row.description, &section)
                        || row.keys.iter().any(|(keys, _)| {
                            shortcut_matches(query, keys, &row.description, &section)
                        })
                })
                .collect();
            if rows.is_empty() {
                continue;
            }
            count += rows.len();
            body = body.child(self.keybind_section_heading(&section));
            for row in rows {
                body = body.child(self.keybind_row(row, cx));
            }
        }
        if count == 0 {
            let theme = &self.theme;
            body = body.child(
                div()
                    .debug_selector(|| "keybinds-empty".into())
                    .py(px(20.))
                    .text_color(rgb(theme.muted))
                    .child(fl!("menu-settings-no-match")),
            );
        }
        let theme = &self.theme;
        (
            body.child(
                div()
                    .py(px(14.))
                    .text_color(rgb(theme.subtext()))
                    .child(fl!("menu-settings-note")),
            ),
            count,
        )
    }

    /// A section's heading, above the rows that list under it.
    fn keybind_section_heading(&self, section: &str) -> Div {
        let font = &self.config.ui;
        div()
            .pt(px(12.))
            .pb(px(6.))
            .text_size(px(font.size * 0.85))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(accent(&self.theme))
            .child(section.to_owned())
    }

    /// One command: every way to reach it, what it is called, and the button
    /// that edits its shortcut.
    fn keybind_row(&self, row: KeybindRow<'_>, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let accent = accent(theme);
        let KeybindRow {
            description,
            info,
            keys,
        } = row;
        div()
            .debug_selector(|| format!("shortcut-{description}"))
            .flex()
            .items_center()
            .gap(px(12.))
            .py(px(7.))
            .border_b_1()
            .border_color(rgb(theme.active))
            .child(
                div()
                    .debug_selector(|| format!("keys-{description}"))
                    .w(relative(0.45))
                    .flex_none()
                    .flex()
                    .flex_wrap()
                    .gap(px(10.))
                    .when(keys.is_empty(), |column| {
                        column.child(
                            div()
                                .text_color(rgb(theme.muted))
                                .child(fl!("menu-settings-no-shortcut")),
                        )
                    })
                    .children(keys.into_iter().map(|(keys, note)| {
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(3.))
                            .child(div().flex().flex_wrap().gap(px(4.)).children(
                                keycaps(&keys).map(|key| {
                                    div()
                                        .flex_none()
                                        .px(px(6.))
                                        .py(px(2.))
                                        .rounded(px(crate::config::corners::SMALL))
                                        .border_1()
                                        .border_color(rgb(theme.active))
                                        .bg(rgb(theme.background))
                                        .text_size(px(font.size * 0.9))
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(key)
                                }),
                            ))
                            .when_some(note, |binding, note| {
                                binding.child(
                                    div()
                                        .text_size(px(font.size * 0.85))
                                        .text_color(rgb(theme.muted))
                                        .child(note),
                                )
                            })
                    })),
            )
            .child(
                div()
                    .debug_selector(|| format!("description-{description}"))
                    .flex_1()
                    .min_w_0()
                    .child(description),
            )
            .when_some(info, |row, info| {
                row.child(
                    div()
                        .id(SharedString::from(format!("edit-shortcut-{}", info.name)))
                        .debug_selector(move || format!("edit-shortcut-{}", info.name))
                        .flex_none()
                        .px(px(6.))
                        .py(px(4.))
                        .cursor_pointer()
                        .text_color(accent)
                        .child(fl!("menu-settings-edit"))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.edit_shortcut(info, window, cx)
                        })),
                )
            })
    }

    /// The page's title, what it is for, and the button that closes it.
    fn keybind_header(&self, cx: &mut Context<Self>) -> Div {
        let theme = &self.theme;
        let font = &self.config.ui;
        let accent = accent(theme);
        div()
            .debug_selector(|| "keybinds-header".into())
            .flex()
            .items_center()
            .flex_none()
            .gap(px(12.))
            .p(px(16.))
            .border_b_1()
            .border_color(rgb(theme.active))
            .child(
                div()
                    .w(px(3.))
                    .h(px(font.size * 2.5))
                    .rounded_full()
                    .bg(accent),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .text_size(px(font.size * 1.35))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(fl!("menu-settings-title")),
                    )
                    .child(
                        div()
                            .text_color(rgb(theme.muted))
                            .child(fl!("menu-settings-subtitle")),
                    ),
            )
            .child(
                div()
                    .id("menu-close")
                    .debug_selector(|| "keybinds-close".into())
                    .flex_none()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded(px(crate::config::corners::CONTROL))
                    .cursor_pointer()
                    .text_color(rgb(theme.muted))
                    .hover(|style| {
                        style
                            .bg(rgb(theme.active))
                            .text_color(rgb(theme.foreground))
                    })
                    .child(fl!("menu-settings-close"))
                    .on_click(cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx))),
            )
    }

    /// The field that narrows the page, and how many rows it has left.
    fn keybind_search_area(&self, count: usize, total: usize, font: &FontConfig) -> Div {
        let theme = &self.theme;
        div()
            .debug_selector(|| "keybinds-search-area".into())
            .flex_none()
            .px(px(16.))
            .py(px(8.))
            .when_some(self.menu.keybinds_search.clone(), |area, search| {
                area.child(search)
            })
            .child(
                div()
                    .debug_selector(|| "keybinds-count".into())
                    .pt(px(4.))
                    .text_size(px(font.size * 0.85))
                    .text_color(rgb(theme.muted))
                    .child(fl!("menu-settings-count", count = count, total = total)),
            )
    }

    /// What the page notes at its foot, under every section.
    fn keybind_footer() -> Div {
        div()
            .debug_selector(|| "keybinds-footer".into())
            .flex_none()
            .px(px(16.))
            .py(px(10.))
            .child(fl!("menu-settings-footer"))
    }
}

/// One row of the Keybinds page: every way to reach the command, what it is
/// called, and the catalog entry its Edit button opens.
struct KeybindRow<'a> {
    description: String,
    info: Option<&'static CommandInfo>,
    keys: Vec<(Cow<'a, str>, Option<String>)>,
}

/// Which section of the page a command's shortcut lists under.
fn keybind_section(command: Command) -> usize {
    match command {
        Command::Workspace
        | Command::Tab
        | Command::SplitRight
        | Command::SplitDown
        | Command::Zoom
        | Command::ClearPane
        | Command::Find
        | Command::FindNext
        | Command::FindPrevious
        | Command::CopyMode
        | Command::EditScrollback
        | Command::ClosePane
        | Command::CloseTab
        | Command::ReopenClosedTab
        | Command::SplitEditor
        | Command::MoveTabPrevious
        | Command::MoveTabNext
        | Command::RenameTab
        | Command::SwapLeft
        | Command::SwapRight
        | Command::SwapUp
        | Command::SwapDown
        | Command::ResizeLeft
        | Command::ResizeRight
        | Command::ResizeUp
        | Command::ResizeDown
        | Command::ResizeMode
        | Command::EqualizeSplits
        | Command::RenamePane
        | Command::MovePane
        | Command::RenameWorkspace => 0,
        Command::NextTab
        | Command::PreviousTab
        | Command::FocusLeft
        | Command::FocusRight
        | Command::FocusUp
        | Command::FocusDown
        | Command::NextPane
        | Command::PreviousPane
        | Command::TabNumber(_)
        | Command::WorkspacePicker
        | Command::LastPane
        | Command::Back
        | Command::Forward
        | Command::PreviousWorkspace
        | Command::NextWorkspace
        | Command::WorkspaceNumber(_)
        | Command::PreviousAgent
        | Command::NextAgent
        | Command::AgentNumber(_)
        | Command::OpenNotificationTarget => 1,
        Command::NewWindow
        | Command::ToggleSidebar
        | Command::ToggleStatusBar
        | Command::FocusNextRegion
        | Command::FocusPreviousRegion
        | Command::ToggleZenMode
        | Command::DevicesOverview
        | Command::NewCoreTerminal
        | Command::MirrorCorePane
        | Command::IncreaseFontSize
        | Command::DecreaseFontSize
        | Command::ResetFontSize
        | Command::Settings
        | Command::Keybinds
        | Command::Sessions
        | Command::Themes
        | Command::Palette
        | Command::ResumePalette
        | Command::Detach
        | Command::Reconnect
        | Command::Quit
        | Command::Logs
        | Command::CheckForUpdates
        | Command::About
        | Command::InstallBrowserSkill
        | Command::ToggleFullScreen
        | Command::CycleWindows
        | Command::CloseWindow
        | Command::ReloadConfig => 2,
    }
}

/// The keycaps of a shortcut, capitalized for display, a prefix chord's
/// keystrokes in turn. `cmd--` splits into `cmd` and a `-` key rather than an
/// empty cap, as does a chord's bare `-`.
/// A custom command's row name: its description, or its ID without one.
fn command_name(command: &ClientShellCommand) -> &str {
    command
        .description
        .as_deref()
        .filter(|description| !description.trim().is_empty())
        .unwrap_or(&command.command_id)
}

/// Why a custom command's binding does not run, beneath its keycaps.
fn reach_note(reach: Reach) -> Option<String> {
    Some(match reach {
        Reach::Runs => return None,
        Reach::Shadowed => fl!("menu-settings-shadowed"),
        Reach::Taken => fl!("menu-settings-taken"),
        Reach::NeedsModifier => fl!("menu-settings-needs-modifier"),
        Reach::NoPrefix => fl!("menu-settings-no-prefix"),
        Reach::OverLimit => fl!("menu-settings-over-limit", limit = 8),
        Reach::Unsupported => fl!("menu-settings-unsupported"),
    })
}

pub(super) fn keycaps(shortcut: &str) -> impl Iterator<Item = String> + '_ {
    shortcut
        .split(' ')
        .flat_map(|keystroke| {
            let (modifiers, key) = match keystroke.strip_suffix('-') {
                Some(modifiers) if modifiers.is_empty() || modifiers.ends_with('-') => {
                    (modifiers, "-")
                }
                _ => keystroke.rsplit_once('-').unwrap_or(("", keystroke)),
            };
            modifiers
                .split('-')
                .filter(|modifier| !modifier.is_empty())
                .chain(std::iter::once(key))
        })
        .map(|key| {
            let mut chars = key.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase())
                .into_iter()
                .chain(chars)
                .collect()
        })
}

fn shortcut_matches(query: &str, keys: &str, description: &str, section: &str) -> bool {
    let query = query.to_lowercase().replace(['-', '+'], " ");
    if query
        .split_whitespace()
        .next()
        .is_some_and(|token| matches!(token, "cmd" | "ctrl" | "alt" | "shift"))
    {
        // A key combination should match keycaps, not letters in an action's name.
        let keys = keys.to_lowercase();
        return query
            .split_whitespace()
            .all(|token| keys.split(['-', '+', ' ']).any(|key| key == token));
    }
    let text = format!("{keys} {description} {section}")
        .to_lowercase()
        .replace(['-', '+'], " ");
    query.split_whitespace().all(|token| text.contains(token))
}

#[cfg(test)]
mod tests;
