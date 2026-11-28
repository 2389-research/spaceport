//! Spaceport's settings: the managed `config.toml`, the local overrides, and
//! the private view settings they are layered over.
//!
//! Loading and saving block. `Config::load`, `Config::load_startup`,
//! `Theme::load`, `Config::save_preference` and `herdr_settings::{load, save}`
//! read and write files: callers MUST run them on a background worker, never
//! in render or input handlers.
// i18n: migrated
use crate::contrast::Contrast;
use crate::daemon::view_settings_path;
use crate::keys::{Binding, PaneKeys};
use crate::notifications::{ClipboardToastSettings, NotificationSettings as SavedNotifications};
use crate::{
    Accent, AgentSkillConfig, AgentsPlacement, Backdrop, BellConfig, ClipboardToast, Error,
    Features, FontConfig, FontSettings, Layout, NotificationConfig, OptionAsAlt, Padding, Result,
    SidebarLayout, SidebarStyle, SoundConfig, SshConfig, StatusBar, StatusIcons, TabLabels,
    TabLayoutMode, ThemeName, Weight, Worktrees, Zen, chat, daemon, devices, font_weight, fonts,
    palette, sidebar_style, status_icons, worktrees, zen,
};
use daemon::{Daemon, daemon_settings};
use devices::{DeviceSettings, MAX_DEVICES};
use palette::PaletteConfig;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub(crate) const DEFAULT_CONFIG: &str = include_str!("../config-gpui.example.toml");
pub(crate) const MANAGED_HEADER: &str = spaceport_client::settings_files::MANAGED_HEADER;
/// Seeds the overrides file on first launch only.
pub(crate) const LOCAL_CONFIG: &str = spaceport_client::settings_files::OVERRIDES_SEED;

/// Shared logical-pixel radii for native-style chrome, independent of the
/// terminal grid. Small badges/keycaps retain a tighter curve than controls.
pub mod corners {
    pub const PANEL: f32 = 12.;
    pub const CONTROL: f32 = 8.;
    pub const SMALL: f32 = 4.;
}

#[derive(Clone, Debug)]
pub struct Config {
    // keep-sorted start
    /// Where the app's own accent color comes from.
    pub accent: Accent,
    /// Where the managed agent skill may go on its own.
    pub agent_skill: AgentSkillConfig,
    /// Where the sidebar lists agents while `show_agents` is on.
    pub agents_placement: AgentsPlacement,
    /// What shows behind the main window's terminals and chrome.
    pub backdrop: Backdrop,
    pub bell: BellConfig,
    /// What chats started from the app run with.
    pub chat: chat::ChatConfig,
    /// Only the keys this GUI's own file sets, so Settings can tell an
    /// override from the Herdr value it shadows.
    pub clipboard_overrides: ClipboardToastSettings,
    pub clipboard_toast: ClipboardToast,
    pub confirm_close_pane: bool,
    pub confirm_close_tab: bool,
    /// How far the app's own marks and labels stand off its chrome.
    pub contrast: Contrast,
    /// Per saved device, by catalog profile ID.
    pub devices: BTreeMap<String, DeviceSettings>,
    /// A bar on the leading edge of each working, blocked, or done row,
    /// whichever status icons are drawn.
    pub edge_rail: bool,
    /// Themes starred in the Color Scheme picker, which lists them first.
    pub favorite_themes: Vec<String>,
    pub features: Features,
    /// Focus the split terminal pane under the pointer without clicking.
    pub focus_follows_mouse: bool,
    /// Prefer Core for new terminal tabs when Core is enabled.
    /// An enabled Core session also supplies terminals when Herdr 1 is absent.
    pub herdr_2_terminals: bool,
    /// Explicit Core opt-in, applied once when the app starts.
    pub herdr_core_enabled: bool,
    /// Font resolution searched for an icon font for the terminal and found
    /// none installed, so Private Use Area glyphs draw as missing-glyph boxes.
    /// Never set when the config names the terminal's `fallback` itself.
    pub icon_font_missing: bool,
    /// Whether a terminal selection stays highlighted, and readable by
    /// selection tools, after it is copied.
    pub keep_selection_after_copy: bool,
    /// The `[keybindings]` table as the file wrote it, kept so the app can
    /// build its keymap and a device's server keys under the same overrides.
    pub keybinding_overrides: BTreeMap<String, Binding>,
    /// The language the app shows: `system`, or a BCP 47 tag (docs/i18n.md).
    pub language: spaceport_i18n::Language,
    pub layout: Layout,
    /// Configured families, sorted, that font resolution found no installed
    /// family for; `diagnostic` reports them.
    pub missing_fonts: Vec<String>,
    /// The terminal's configured weight, when its family has no face at it.
    pub missing_weight: Option<font_weight::MissingWeight>,
    pub notification_overrides: SavedNotifications,
    pub notifications: NotificationConfig,
    pub option_as_alt: OptionAsAlt,
    pub palette: PaletteConfig,
    /// The `[pane_keys]` table as the file wrote it, for the same reason.
    pub pane_keys: PaneKeys,
    pub quiet_after: crate::QuietAfter,
    pub show_agents: bool,
    /// The "Agents" title row above the agents list, which also holds the sort
    /// toggle. The sort then follows Herdr's `ui.agent_panel_sort` unless it
    /// was toggled earlier, unless a plugin sets the agent view, which keeps the
    /// header visible. Set in Settings > Sidebar or with
    /// `spaceport config set show_agents_header false`.
    pub show_agents_header: bool,
    /// The mark before a device's name in the sidebar, in the layouts that
    /// draw one: where the device runs and how it is reached.
    pub show_device_icons: bool,
    /// Ports each workspace listens on, in the sidebar and the status bar.
    pub show_listening_ports: bool,
    /// Latency and loss from this machine, by timed TCP connects, in the status bar.
    pub show_network_quality: bool,
    /// The repository owner's avatar, or the GitHub mark, before a space's name.
    pub show_repository_icons: bool,
    /// The search field at the top of the sidebar's spaces list.
    pub show_sidebar_search: bool,
    /// Draw minimize, maximize, and close buttons in the menu bar on Linux.
    /// When true the window needs no title bar from the window manager.
    pub show_window_controls: bool,
    pub sidebar: FontConfig,
    /// Sidebar rows from private view settings, with defaults when invalid.
    pub sidebar_layout: SidebarLayout,
    /// Spacing overrides, host colours, and selection marking for the sidebar.
    pub sidebar_style: SidebarStyle,
    /// Wheel scrollback draws panes between rows, following the OS deltas.
    pub smooth_scroll: bool,
    /// How loud agent notification sounds play.
    pub sound: SoundConfig,
    /// SSH hosts whose host keys the user chose not to verify.
    pub ssh: SshConfig,
    /// Which status bar items show, and how compactly.
    pub status_bar: StatusBar,
    /// The set agent status is drawn with.
    pub status_icons: StatusIcons,
    /// What a tab with no name of its own is called.
    pub tab_labels: TabLabels,
    /// How tab strips draw their tabs, from View > Tab Layout.
    pub tab_layout: TabLayoutMode,
    pub tabs: FontConfig,
    pub terminal: FontConfig,
    /// Blank space around every terminal's cells.
    pub terminal_padding: Padding,
    pub theme: String,
    pub ui: FontConfig,
    /// Keys the file names that this build does not know, sorted. They are
    /// ignored, as Herdr ignores its own, so a config written by a newer
    /// build or with a typo still loads; `diagnostic` reports them.
    pub unknown_keys: Vec<String>,
    pub usage: spaceport_usage::UsageConfig,
    /// The private view settings' `[keys]` table as written, kept so the app
    /// can build its keymap from the same defaults this config was layered
    /// over. Not a value this crate resolves: keystrokes are the app's.
    pub view_keys: Option<toml::Table>,
    /// How linked worktrees are marked apart from their main checkout.
    pub worktrees: Worktrees,
    /// What Toggle Zen Mode does beyond hiding the chrome.
    pub zen: Zen,
    // keep-sorted end
}

impl Default for Config {
    fn default() -> Self {
        let fonts::DefaultFonts {
            monospace,
            sans: ui,
            ..
        } = fonts::PLATFORM_FONTS;
        let font = |family: &str, size| FontConfig {
            family: family.into(),
            size,
            fallbacks: None,
            line_height_multiple: None,
            weight: Weight::NORMAL,
        };
        Self {
            // keep-sorted start
            accent: Accent::default(),
            agent_skill: AgentSkillConfig::default(),
            agents_placement: AgentsPlacement::default(),
            backdrop: Backdrop::default(),
            bell: BellConfig::default(),
            chat: chat::ChatConfig::default(),
            clipboard_overrides: ClipboardToastSettings::default(),
            clipboard_toast: ClipboardToast::default(),
            confirm_close_pane: true,
            confirm_close_tab: true,
            contrast: Contrast::default(),
            devices: BTreeMap::new(),
            edge_rail: false,
            favorite_themes: Vec::new(),
            features: Features::default(),
            focus_follows_mouse: true,
            herdr_2_terminals: false,
            herdr_core_enabled: false,
            icon_font_missing: false,
            keep_selection_after_copy: true,
            keybinding_overrides: BTreeMap::new(),
            language: spaceport_i18n::Language::default(),
            layout: Layout::default(),
            missing_fonts: Vec::new(),
            missing_weight: None,
            notification_overrides: SavedNotifications::default(),
            notifications: NotificationConfig::default(),
            option_as_alt: OptionAsAlt::default(),
            palette: PaletteConfig::default(),
            pane_keys: PaneKeys::new(),
            quiet_after: crate::QuietAfter::default(),
            show_agents: true,
            show_agents_header: true,
            show_device_icons: true,
            show_listening_ports: true,
            show_network_quality: false,
            show_repository_icons: true,
            show_sidebar_search: true,
            show_window_controls: cfg!(target_os = "linux"),
            sidebar: font(monospace, 12.0),
            sidebar_layout: SidebarLayout::default(),
            sidebar_style: SidebarStyle::default(),
            smooth_scroll: true,
            sound: SoundConfig::default(),
            ssh: SshConfig::default(),
            status_bar: StatusBar::default(),
            status_icons: StatusIcons::default(),
            tab_labels: TabLabels::default(),
            tab_layout: TabLayoutMode::default(),
            // Tabs are terminal chrome, so they read in the monospace face the
            // sidebar and terminal use, as they do in the reference UI.
            tabs: font(monospace, 12.0),
            terminal: font(monospace, 14.0),
            terminal_padding: Padding::default(),
            theme: "Default".into(), // i18n: data
            ui: font(ui, 12.0),
            unknown_keys: Vec::new(),
            usage: spaceport_usage::UsageConfig::default(),
            view_keys: None,
            worktrees: Worktrees::default(),
            zen: Zen::default(),
            // keep-sorted end
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Settings {
    // keep-sorted start
    accent: Accent,
    agent_skill: AgentSkillConfig,
    agents_placement: AgentsPlacement,
    backdrop: Backdrop,
    bell: BellConfig,
    chat: chat::ChatConfig,
    clipboard_toast: ClipboardToastSettings,
    confirm_close_pane: Option<bool>,
    confirm_close_tab: Option<bool>,
    contrast: Contrast,
    devices: BTreeMap<String, DeviceSettings>,
    edge_rail: Option<bool>,
    favorite_themes: Vec<String>,
    features: Features,
    focus_follows_mouse: Option<bool>,
    herdr_2_terminals: Option<bool>,
    herdr_core_enabled: Option<bool>,
    keep_selection_after_copy: Option<bool>,
    keybindings: BTreeMap<String, Binding>,
    language: Option<spaceport_i18n::Language>,
    layout: Layout,
    notifications: SavedNotifications,
    option_as_alt: OptionAsAlt,
    palette: PaletteConfig,
    pane_keys: PaneKeys,
    quiet_after: crate::QuietAfter,
    show_agents: Option<bool>,
    show_agents_header: Option<bool>,
    show_device_icons: Option<bool>,
    show_listening_ports: Option<bool>,
    show_network_quality: Option<bool>,
    show_repository_icons: Option<bool>,
    show_sidebar_search: Option<bool>,
    show_window_controls: Option<bool>,
    sidebar: sidebar_style::SidebarSettings,
    smooth_scroll: Option<bool>,
    sound: SoundConfig,
    ssh: SshConfig,
    status_bar: StatusBar,
    status_icons: Option<status_icons::SavedIcons>,
    tab_labels: TabLabels,
    tab_layout: TabLayoutMode,
    tabs: FontSettings,
    terminal: FontSettings,
    theme: Option<String>,
    ui: FontSettings,
    usage: spaceport_usage::UsageConfig,
    worktrees: worktrees::WorktreeSettings,
    zen: zen::ZenSettings,
    // keep-sorted end
}

/// Windows sets `USERPROFILE` rather than `HOME`, and upstream Herdr reads both.
pub fn home() -> Result<PathBuf> {
    spaceport_client::local_path::home_dir().ok_or(Error::MissingHome)
}

/// The platform configuration root, including the login shell's
/// `XDG_CONFIG_HOME` for a desktop launch that did not inherit it.
pub(crate) fn config_root() -> Result<PathBuf> {
    use spaceport_client::local_path::PathError;
    spaceport_client::local_path::config_root().map_err(|error| match error {
        PathError::MissingHome => Error::MissingHome,
        PathError::RelativeConfigRoot => Error::RelativeConfigRoot,
    })
}

pub(crate) fn app_config_path(root: &Path) -> PathBuf {
    root.join("spaceport/config.toml")
}

impl Config {
    pub fn path() -> Result<PathBuf> {
        Ok(app_config_path(&config_root()?))
    }

    pub fn local_path() -> Result<PathBuf> {
        Ok(Self::path()?.with_extension("local.toml"))
    }

    pub fn load() -> Result<Self> {
        Self::load_path(
            &Self::path()?,
            &view_settings_path(spaceport_client::local_var),
        )
    }

    /// First-frame settings only: no lock, migration, writes, or fsync. The
    /// background load performs maintenance after the window has appeared.
    pub fn load_startup() -> Result<Self> {
        Self::load_startup_path(
            &Self::path()?,
            &view_settings_path(spaceport_client::local_var),
        )
    }

    pub(crate) fn load_startup_path(path: &Path, daemon: &Path) -> Result<Self> {
        let (text, source) = Self::overrides_text(path)?;
        Self::parse_layers([DEFAULT_CONFIG, &text], &daemon_settings(daemon))
            .map_err(|error| error.at_path(&source))
            .map(|config| config.with_desktop_theme(&text))
    }

    /// The user's overrides as the app reads them now, and the file they
    /// come from, without writing anything.
    pub(super) fn overrides_text(path: &Path) -> Result<(String, PathBuf)> {
        let local = path.with_extension("local.toml");
        Ok(match fs::read_to_string(&local) {
            Ok(text) => (text, local),
            // Without overrides or a personal config to migrate, maintenance
            // will seed the first-launch overrides; show them from frame one.
            Err(error) if error.kind() == ErrorKind::NotFound => match fs::read_to_string(path) {
                Ok(text) if text.lines().next() != Some(MANAGED_HEADER) => (text, path.to_owned()),
                Ok(_) => (LOCAL_CONFIG.into(), local),
                Err(error) if error.kind() == ErrorKind::NotFound => (LOCAL_CONFIG.into(), local),
                Err(error) => return Err(Error::from(error).at_path(path)),
            },
            Err(error) => return Err(Error::from(error).at_path(&local)),
        })
    }

    /// `daemon` is Spaceport's own `view.toml`, the file whose `[keys]`,
    /// `ui.toast.clipboard` and `ui.sidebar` this GUI also honors. The Settings
    /// window writes its own keys there too; a missing file is normal.
    pub(crate) fn load_path(path: &Path, daemon: &Path) -> Result<Self> {
        let base = daemon_settings(daemon);
        let (_lock, local) = Self::prepare_files(path)?;
        let text =
            fs::read_to_string(&local).map_err(|error| Error::from(error).at_path(&local))?;
        // Validate the override independently so bad types/unknown keys cannot
        // disappear inside the merge. Empty arrays explicitly replace defaults.
        Self::parse_over(&text, &base).map_err(|error| error.at_path(&local))?;
        Self::parse_layers([DEFAULT_CONFIG, &text], &base)
            .map_err(|error| error.at_path(&local))
            .map(|config| config.with_desktop_theme(&text))
    }

    /// Without a home directory there is no desktop theme to find, and the
    /// managed default stays. Tests load temporary files and must not follow
    /// the desktop of the machine running them; `follow_omarchy` is tested
    /// against a home of its own.
    pub(crate) fn with_desktop_theme(mut self, overrides: &str) -> Self {
        if !cfg!(test)
            && let Ok(home) = home()
        {
            self.follow_omarchy(overrides, &home);
        }
        self
    }

    /// The GUI file on its own, with nothing layered under it: the shape the
    /// tests below read, since loading also consults the daemon's config.
    #[cfg(test)]
    pub(crate) fn parse(text: &str) -> Result<Self> {
        Self::parse_over(text, &Daemon::default())
    }

    /// `base` is what the daemon's own config asked for, which every key this
    /// file names overrides.
    pub(crate) fn parse_over(text: &str, base: &Daemon) -> Result<Self> {
        Self::parse_layers([text], base)
    }

    /// The files layered as loading layers them, later ones over earlier.
    pub(crate) fn merged<'a>(texts: impl IntoIterator<Item = &'a str>) -> Result<toml::Value> {
        let mut builder = config_loader::Config::builder();
        for text in texts {
            builder = builder.add_source(config_loader::File::from_str(
                text,
                config_loader::FileFormat::Toml,
            ));
        }
        let loaded = builder.build()?;
        // Config's typed deserializer coerces strings/numbers. Preserve TOML
        // types so existing strict font and theme validation remains intact.
        Ok(loaded.try_deserialize()?)
    }

    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 139 lines")]
    pub(crate) fn parse_layers<'a>(
        texts: impl IntoIterator<Item = &'a str>,
        base: &Daemon,
    ) -> Result<Self> {
        let value = Self::merged(texts)?;
        let mut unknown_keys = Vec::new();
        let mut settings: Settings = serde_ignored::deserialize(value, |path| {
            unknown_keys.push(path.to_string());
        })?;
        settings.keybindings.retain(|name, _| {
            let known = spaceport_commands::COMMANDS
                .iter()
                .any(|info| info.name == name);
            if !known {
                unknown_keys.push(format!("keybindings.{name}"));
            }
            known
        });
        unknown_keys.extend(settings.usage.retain_known());
        // Only catalog profile IDs name a device; anything else is ignored
        // and reported like any other unknown key.
        settings.devices.retain(|id, _| {
            let known = spaceport_client::valid_profile_id(id);
            if !known {
                unknown_keys.push(format!("devices.{id}"));
            }
            known
        });
        // Only the terminal grid takes a line height or weight. The other
        // faces draw fixed chrome whose emphasis sets its own weights, so
        // there both are ignored like any other unknown key.
        unknown_keys.extend(
            [
                ("sidebar", &mut settings.sidebar.font),
                ("tabs", &mut settings.tabs),
                ("ui", &mut settings.ui),
            ]
            .into_iter()
            .flat_map(|(name, face)| face.reject_terminal_keys(name)),
        );
        // An old account table must not silently become accepted inert data:
        // the app no longer owns GitHub sign-ins, and settings must say so.
        if unknown_keys
            .iter()
            .any(|key| key == "github" || key.starts_with("github."))
        {
            return Err(Error::RemovedGitHub);
        }
        if unknown_keys
            .iter()
            .any(|key| key == "updates" || key.starts_with("updates."))
        {
            return Err(Error::RemovedUpdates);
        }
        unknown_keys.sort();
        unknown_keys.dedup();
        let mut config = Self {
            unknown_keys,
            ..Self::default()
        };
        settings.chat.check()?;
        config.chat = settings.chat;
        config.favorite_themes = settings.favorite_themes;
        // A hand-edited list may name a theme twice; it is starred once.
        let mut starred = std::collections::HashSet::new();
        config
            .favorite_themes
            .retain(|name| starred.insert(name.clone()));
        config.features = settings.features;
        config.notification_overrides = settings.notifications;
        config.notifications = settings
            .notifications
            .resolve(NotificationConfig::default());
        config.clipboard_overrides = settings.clipboard_toast;
        config.clipboard_toast = settings.clipboard_toast.resolve(base.clipboard_toast);
        config.bell = settings.bell;
        config.sound = settings.sound;
        config.sidebar_layout = base.sidebar_layout.clone();
        config.view_keys = base.keys.clone();
        let (padding_x, padding_y) = settings.terminal.take_padding();
        config.terminal_padding = Padding::resolve(
            padding_x.as_ref(),
            padding_y.as_ref(),
            settings.layout.legacy_sidebar_gap,
        )?;
        config.layout = settings.layout;
        config.language = settings.language.unwrap_or_default();
        config.sidebar_style = settings.sidebar.style()?;
        config.keybinding_overrides = settings.keybindings;
        config.pane_keys = settings.pane_keys;
        if settings.devices.len() > MAX_DEVICES {
            return Err(Error::TooManyDevices(MAX_DEVICES));
        }
        for (id, device) in &settings.devices {
            device.validate(id)?;
        }
        config.devices = settings.devices;
        settings.palette.validate()?;
        config.palette = settings.palette;
        if let Some(theme) = settings.theme {
            if theme.trim().is_empty() {
                return Err(Error::EmptyTheme);
            }
            ThemeName::parse(&theme)?;
            config.theme = theme;
        }
        // keep-sorted start
        config.confirm_close_pane = settings.confirm_close_pane.unwrap_or(true);
        config.confirm_close_tab = settings.confirm_close_tab.unwrap_or(true);
        config.focus_follows_mouse = settings.focus_follows_mouse.unwrap_or(true);
        config.show_agents = settings.show_agents.unwrap_or(true);
        config.show_agents_header = settings.show_agents_header.unwrap_or(true);
        config.show_device_icons = settings.show_device_icons.unwrap_or(true);
        config.show_listening_ports = settings.show_listening_ports.unwrap_or(true);
        config.show_network_quality = settings.show_network_quality.unwrap_or(false);
        config.show_repository_icons = settings.show_repository_icons.unwrap_or(true);
        config.show_sidebar_search = settings.show_sidebar_search.unwrap_or(true);
        config.show_window_controls = settings
            .show_window_controls
            .unwrap_or(cfg!(target_os = "linux"));
        config.smooth_scroll = settings.smooth_scroll.unwrap_or(true);
        // keep-sorted end
        config.status_bar = settings.status_bar;
        config.tab_labels = settings.tab_labels;
        config.tab_layout = settings.tab_layout;
        settings.ssh.check()?;
        config.ssh = settings.ssh;
        let (status_icons, legacy_rail) = status_icons::SavedIcons::resolve(settings.status_icons);
        config.status_icons = status_icons;
        config.edge_rail = settings.edge_rail.unwrap_or(legacy_rail);
        config.zen = settings.zen.try_into()?;
        config.worktrees = settings.worktrees.resolve()?;
        settings.backdrop.validate()?;
        config.backdrop = settings.backdrop;
        config.accent = settings.accent;
        config.contrast = settings.contrast;
        config.agents_placement = settings.agents_placement;
        config.quiet_after = settings.quiet_after;
        config.agent_skill = settings.agent_skill;
        config.usage = settings.usage;
        config.option_as_alt = settings.option_as_alt;
        config.keep_selection_after_copy = settings.keep_selection_after_copy.unwrap_or(true);
        config.herdr_2_terminals = settings.herdr_2_terminals.unwrap_or(false);
        config.herdr_core_enabled = settings.herdr_core_enabled.unwrap_or(false);
        for (name, font, settings) in [
            ("sidebar", &mut config.sidebar, settings.sidebar.font),
            ("tabs", &mut config.tabs, settings.tabs),
            ("terminal", &mut config.terminal, settings.terminal),
            ("ui", &mut config.ui, settings.ui),
        ] {
            settings.apply(name, font)?;
        }
        Ok(config)
    }
}
