//! Spaceport's settings values: the typed tables the managed config file and
//! Herdr's shared settings carry, with their defaults and validation, and the
//! contrast, font-weight and padding rules the app reads them with.
//!
//! Everything here is a value. The app keeps what acts on one — spawning a
//! watcher, installing an SSH hook, painting — and its own messages.

pub mod accent;
pub mod agent_skill;
pub mod agents_placement;
pub mod bitmap_fonts;
pub mod cjk_fonts;
pub mod contrast;
mod error;
pub mod features;
pub mod font_weight;
pub mod layout;
pub mod option_as_alt;
pub mod padding;
pub mod sidebar;
pub mod status_bar;
pub mod status_icons;
pub mod tab_labels;
pub mod tab_layout;
mod toml_value;

pub use accent::Accent;
pub use agent_skill::{AgentSkillConfig, SkillInstall};
pub use agents_placement::AgentsPlacement;
pub use error::{Error, Result, ThemeParseError};
pub use features::{Feature, Features};
pub use font_weight::{MissingWeight, Weight};
pub use layout::{Density, Layout, LayoutMode, Style};
pub use option_as_alt::{OptionAsAlt, OptionKeys};
pub use padding::{Axis, MAX_PADDING, Padding, Side, Sides};
pub use sidebar::{
    AgentLayout, AgentToken, Rows, SidebarLayout, SpaceLayout, SpaceToken, TokenStyle, parse_hex,
};
pub use status_bar::StatusBar;
pub use status_icons::StatusIcons;
pub use tab_labels::TabLabels;
pub use tab_layout::TabLayoutMode;
