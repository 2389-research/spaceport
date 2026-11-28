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
    config::{Config, ThemeName},
    controls::{COMMANDS, Command},
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::ClientShellCommand;
use spaceport_keymap::Reach;
use spaceport_menu::keybinds::render::{self, Action, Hosts, Words};
use spaceport_menu::keybinds::{self, Binding, Row, Section};
use spaceport_settings::Appearance;

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
        let query = self
            .menu
            .keybinds_search
            .as_ref()
            .map(|search| search.read(cx).text().to_owned())
            .unwrap_or_default();
        let mut sections = self.keybind_sections();
        let total = keybinds::total(&sections);
        let count = keybinds::filter(&mut sections, &query);
        let words = Words {
            close: fl!("menu-settings-close"),
            count: fl!("menu-settings-count", count = count, total = total),
            edit: fl!("menu-settings-edit"),
            empty: fl!("menu-settings-no-match"),
            footer: fl!("menu-settings-footer"),
            no_shortcut: fl!("menu-settings-no-shortcut"),
            note: fl!("menu-settings-note"),
            subtitle: fl!("menu-settings-subtitle"),
            title: fl!("menu-settings-title"),
        };
        let text = crate::menu::chrome::text(&self.config.ui);
        render::panel(
            Hosts {
                accent: accent(&self.theme),
                editor: self
                    .render_shortcut_editor(cx)
                    .map(IntoElement::into_any_element),
                scroll: &self.menu.keybinds_scroll,
                search: self
                    .menu
                    .keybinds_search
                    .clone()
                    .map(IntoElement::into_any_element),
                sections: &sections,
                text: &text,
                theme: &self.theme,
                words: &words,
            },
            |action, button| match action {
                Action::Close => button
                    .on_click(cx.listener(|this, _, window, cx| this.dismiss_menu(window, cx))),
                Action::Edit(name) => button.on_click(cx.listener(move |this, _, window, cx| {
                    if let Some(info) = COMMANDS.iter().find(|info| info.name == name) {
                        this.edit_shortcut(info, window, cx);
                    }
                })),
            },
        )
    }

    /// Every row the page lists, by section: the catalog's commands with the
    /// keys the keymap gives them, then the daemon's own commands.
    fn keybind_sections(&self) -> Vec<Section> {
        let paste = paste_label();
        let headed = |heading: String, rows: Vec<Row>| Section { heading, rows };
        let mut sections = vec![
            headed(fl!("menu-settings-workspaces"), Vec::new()),
            headed(fl!("menu-settings-navigation"), Vec::new()),
            headed(
                fl!("menu-settings-application"),
                vec![Row {
                    description: paste.clone(),
                    bindings: vec![Binding {
                        keys: crate::actions::EDIT_KEYS[2].to_owned(),
                        note: None,
                    }],
                    edit: None,
                }],
            ),
            headed(fl!("menu-settings-commands"), Vec::new()),
        ];
        for info in COMMANDS {
            if !self.core_enabled() && crate::core_feature::command_requires_core(info.command) {
                continue;
            }
            let mut keys: Vec<String> = self
                .keymap()
                .shortcuts(info.command)
                .map(str::to_owned)
                .collect();
            if info.command == Command::Palette && self.config.palette.double_shift {
                keys.push("shift shift".to_owned()); // i18n: data
            }
            let section = keybind_section(info.command);
            sections[section].rows.push(Row {
                description: crate::i18n::command(info),
                bindings: keys
                    .into_iter()
                    .map(|keys| Binding { keys, note: None })
                    .collect(),
                edit: Some(info.name),
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
            sections[3]
                .rows
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
                        Row {
                            description,
                            bindings: bindings
                                .into_iter()
                                .map(|binding| Binding {
                                    keys: binding.label,
                                    note: reach_note(binding.reach),
                                })
                                .collect(),
                            edit: None,
                        }
                    },
                ));
        }
        sections
    }
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

#[cfg(test)]
mod tests;
