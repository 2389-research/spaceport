//! Worktree notes and scripts targeted at the caller's checkout.
//! Untrusted scripts still require confirmation in the app.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkOp {
    /// Every worktree note, most recently edited first.
    Notes,
    /// The note on the caller's checkout.
    Note,
    /// Replace the caller's checkout's note; empty text clears it.
    SetNote { text: String },
    /// Whether the caller's workspace is muted.
    MuteStatus,
    /// Mute or unmute the caller's workspace.
    Mute { muted: bool },
    /// Run one of the caller's repository's worktree scripts in a new tab.
    /// An untrusted script still asks the user first.
    Script { kind: Script },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Script {
    Setup,
    Run,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteInfo {
    pub host: String,
    pub repository: String,
    pub branch: String,
    pub text: String,
}

/// The caller's workspace's mute.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MuteInfo {
    pub host: String,
    pub workspace: String,
    /// The workspace itself is muted.
    pub muted: bool,
    /// Its whole host is muted, which silences it either way.
    pub host_muted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reply {
    /// The most recent notes that fit one answer; `omitted` older ones did not.
    Notes {
        notes: Vec<NoteInfo>,
        #[serde(default)]
        omitted: usize,
    },
    Note {
        note: Option<NoteInfo>,
    },
    Mute {
        mute: MuteInfo,
    },
}

#[cfg(test)]
mod tests;
