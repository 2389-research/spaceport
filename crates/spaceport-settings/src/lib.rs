//! Spaceport's settings: the typed tables the managed config file and Herdr's
//! shared settings carry, with their defaults, validation and load.
//!
//! Reading and writing them blocks. `Config::load`, `Config::load_startup`,
//! `Theme::load`, `Config::save_preference` and `herdr_settings::{load, save}`
//! touch files: callers MUST run them on a background worker, never in render
//! or input handlers.
//!
//! Everything here is a value or its file, and the policy over those values
//! that needs no window: the Settings pages' sections and rows, the drafts an
//! unsaved edit lives in, and the bounded queue their saves run through. The
//! app keeps what acts on one — spawning a watcher, installing an SSH hook,
//! painting, building the keymap from the raw tables, wording a diagnostic.

// 2026-11-27 quality gate: 3 functions here are over clippy.toml's 100-line
// ceiling and 1 over its cognitive-complexity ceiling. Split them, then delete
// this with the crate's entry in docs/notes-quality-gate-sweep.md.
#![allow(clippy::cognitive_complexity, clippy::too_many_lines)]

pub mod accent;
pub mod agent_skill;
pub mod agents_placement;
pub mod backdrop;
pub mod bitmap_fonts;
pub mod catalog;
pub mod chat;
pub mod cjk_fonts;
pub mod config;
pub mod contrast;
pub mod daemon;
pub mod devices;
pub mod drafts;
pub mod edit;
mod error;
pub mod favorite_themes;
pub mod features;
pub mod fields;
pub mod files;
pub mod font_weight;
pub mod fonts;
pub mod herdr_settings;
pub mod keybindings;
pub mod keys;
pub mod layout;
mod localize;
pub mod notifications;
pub mod omarchy;
pub mod option_as_alt;
pub mod padding;
pub mod palette;
pub mod preferences;
mod quiet_after;
pub mod save_queue;
pub mod section;
pub mod sidebar;
pub mod sidebar_draft;
pub mod sidebar_style;
pub mod ssh_hosts;
pub mod status_bar;
pub mod status_icons;
pub mod tab_labels;
pub mod tab_layout;
#[cfg(test)]
mod tests;
pub mod theme;
mod toml_value;
pub mod worktrees;
pub mod zen;

pub use accent::Accent;
pub use agent_skill::{AgentSkillConfig, SkillInstall};
pub use agents_placement::AgentsPlacement;
pub use backdrop::{Backdrop, Material};
pub use chat::{ChatLauncher, ChatLayoutMode};
pub use config::{Config, corners, home};
pub(crate) use config::{DEFAULT_CONFIG, LOCAL_CONFIG, config_root};
pub use daemon::view_settings_path;
pub(crate) use daemon::{Daemon, daemon_settings};
pub use devices::{DeviceSettings, KeybindingSource};
pub use drafts::Drafts;
pub use edit::{Layer, parse_value};
pub use error::{BackdropProblem, Error, Result, ThemeParseError};
pub use features::{Feature, Features};
pub(crate) use files::write_config;
pub use font_weight::{MissingWeight, Weight};
pub use fonts::{FONT_SIZE_RANGE, FONT_SIZE_STEP, FontConfig, FontFace, FontSettings};
pub use herdr_settings::{AgentSort, Modifiers};
pub use keys::{Binding, PaneKeys};
pub use layout::{Density, Layout, LayoutMode, Style};
pub use notifications::{
    BellConfig, ClipboardToast, ClipboardToastPosition, NotificationConfig, NotificationDelivery,
    NotificationSettings, SoundConfig,
};
pub use option_as_alt::{OptionAsAlt, OptionKeys};
pub use padding::{Axis, MAX_PADDING, Padding, Side, Sides};
pub use preferences::Preference;
pub use quiet_after::QuietAfter;
pub use sidebar::{
    AgentLayout, AgentToken, Rows, SidebarLayout, SpaceLayout, SpaceToken, TokenStyle,
};
pub use sidebar_style::{DeviceFooter, SelectMode, SidebarOverrides, SidebarStyle, parse_hex};
pub use ssh_hosts::SshConfig;
pub use status_bar::StatusBar;
pub use status_icons::StatusIcons;
pub use tab_labels::TabLabels;
pub use tab_layout::TabLayoutMode;
pub use theme::{FOLLOW_HERDR, Theme, ThemeName, mix};
pub use worktrees::{Surface, Worktrees};
pub use zen::{Zen, ZenWidth};
