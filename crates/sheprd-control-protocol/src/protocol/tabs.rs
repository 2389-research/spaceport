//! The app's own tabs and Devices overview, which Herdr does not know about.
use serde::{Deserialize, Serialize};

/// How long the window may take to switch to the caller's workspace before
/// opening a tab there.
pub const ARRIVE_WAIT_SECONDS: u64 = 5;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum TabOp {
    List {},
    Focus { id: u64 },
    Close { id: u64 },
    Devices,
}

impl TabOp {
    pub fn wait_seconds(&self) -> u64 {
        match self {
            Self::Devices => ARRIVE_WAIT_SECONDS,
            Self::List {} | Self::Focus { .. } | Self::Close { .. } => 0,
        }
    }
}

/// A step in or out, back to 100%, or a zoom of its own, in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "snake_case", deny_unknown_fields)]
pub enum ZoomChange {
    In,
    Out,
    Reset,
    Percent { percent: u16 },
}

impl TabKind {
    /// The kind as `--json` spells it, for the text listing.
    pub fn name(self) -> &'static str {
        match self {
            Self::Blank => "blank",
            Self::Web => "web",
            Self::File => "file",
            Self::Devices => "devices",
            Self::CoreTerminal => "core_terminal",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabKind {
    Blank,
    Web,
    File,
    Devices,
    /// A herdr-core (Herdr 2) terminal the app draws.
    CoreTerminal,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: u64,
    pub kind: TabKind,
    pub title: String,
    /// The address, file, or checkout the tab shows.
    pub location: Option<String>,
    pub workspace: String,
    /// The host the workspace belongs to: a local socket or an SSH host.
    pub host: String,
    /// The pane that opened the tab, when an agent did.
    pub opened_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reply {
    Tabs { tabs: Vec<TabInfo> },
    Opened { id: u64, workspace: String },
}

#[cfg(test)]
mod tests;
