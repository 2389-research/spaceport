//! The app, its windows, and their chrome.
use serde::{Deserialize, Serialize};

/// What the window shows, picked on the bar. The bar lists them in
/// declaration order, so this order is what users see.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActivityPanel {
    #[default]
    Workspaces,
    Overview,
    Automations,
    Usage,
}

impl ActivityPanel {
    pub const ALL: &[Self] = &[
        Self::Workspaces,
        Self::Overview,
        Self::Automations,
        Self::Usage,
    ];

    /// The name the CLI and the preferences file use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Workspaces => "workspaces",
            Self::Automations => "automations",
            Self::Usage => "usage",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|panel| panel.name() == name)
    }
}

/// `on`, `off`, or `toggle`, as several settings accept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Switch {
    On,
    Off,
    #[default]
    Toggle,
}

impl Switch {
    /// The state after applying this to `current`.
    pub fn apply(self, current: bool) -> bool {
        match self {
            Self::On => true,
            Self::Off => false,
            Self::Toggle => !current,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum AppOp {
    /// Bring the app and its window forward.
    Focus,
    Hide,
    /// Quit the app; Herdr and its terminals keep running.
    Quit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum WindowOp {
    List,
    /// Another window on the caller-selected launch target.
    New {},
    Focus,
    Fullscreen {
        switch: Switch,
    },
    /// Reports the window's size, or resizes its content when given one.
    /// GPUI cannot move a window, so there is no position to set.
    Size {
        #[serde(default)]
        size: Option<Size>,
    },
    /// Detach from the selected host; it keeps running.
    Detach,
    Reconnect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "snake_case", deny_unknown_fields)]
pub enum FontSize {
    Larger,
    Smaller,
    /// Back to the size the config file sets.
    Reset,
    Points {
        points: u16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum UiOp {
    Sidebar {
        switch: Switch,
    },
    StatusBar {
        switch: Switch,
    },
    FontSize {
        size: FontSize,
    },
    /// Shows an activity bar section.
    Panel {
        panel: ActivityPanel,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum CommandOp {
    /// A command by its `[keybindings]` name, as the palette runs it.
    Run { name: String },
}

/// What `status` reports about the running app.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub version: String,
    pub protocol: u32,
    pub pid: u32,
    /// Whether pages show in tabs; otherwise they open in the system browser.
    pub browser_tabs: bool,
    pub windows: Vec<WindowInfo>,
    #[serde(default)]
    pub core_catalog: CoreCatalog,
}

/// What this client has observed from Core's machine catalog in its current epoch.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreCatalog {
    pub epoch: Option<u64>,
    pub state: CoreCatalogState,
    /// `None` means no current catalog, unlike a current catalog of zero machines.
    pub machine_count: Option<usize>,
    /// A stable class only; Core's untrusted diagnostic text stays out of status.
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreCatalogState {
    #[default]
    NotStarted,
    Synchronizing,
    Current,
    Disconnected,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub active: bool,
    pub fullscreen: bool,
    pub width: f32,
    pub height: f32,
    /// The selected legacy workspace, only while a legacy host is shown.
    pub workspace: Option<String>,
    /// The selected Core presentation group, even if its document is stale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub core_workspace: Option<CoreWorkspaceInfo>,
    pub hosts: Vec<HostInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreWorkspaceInfo {
    pub hub_core_id: String,
    pub machine_id: Option<String>,
    pub document_name: String,
    pub group_id: String,
    /// Whether the exact group is available in a fresh observation of its hub.
    pub current: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostInfo {
    pub id: String,
    pub label: String,
    /// "online", "connecting", "reconnecting", "detached", "disabled", ...
    pub status: String,
    pub selected: bool,
    /// Its notifications are muted as a whole.
    #[serde(default)]
    pub muted: bool,
    /// What groups it in the sidebar.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reply {
    Windows { windows: Vec<WindowInfo> },
    Size { width: f32, height: f32 },
    FontSize { points: f32 },
}

#[cfg(test)]
mod tests;
