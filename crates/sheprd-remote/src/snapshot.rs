//! ABOUTME: Decodes the Herdr CLI pane, agent session, and process views.
//! ABOUTME: Ignores unknown wire fields while keeping exact pane and process identities.

use serde::Deserialize;

/// `{"id": .., "result": T}`, as every `herdr` API subcommand prints it.
#[derive(Debug, Deserialize)]
pub struct Envelope<T> {
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotResult {
    pub snapshot: HostSnapshot,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HostSnapshot {
    pub panes: Vec<Pane>,
}

impl HostSnapshot {
    pub fn pane(&self, id: &str) -> Option<&Pane> {
        self.panes.iter().find(|pane| pane.pane_id == id)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pane {
    pub pane_id: String,
    pub tab_id: String,
    pub cwd: Option<String>,
    pub foreground_cwd: Option<String>,
    pub agent: Option<String>,
    pub agent_session: Option<AgentSession>,
    /// What the daemon reports the pane's agent doing.
    #[serde(default, rename = "agent_status", deserialize_with = "pane_status")]
    pub status: PaneStatus,
}

/// What a pane's agent is doing, as far as a caller here needs to tell.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PaneStatus {
    Done,
    Idle,
    Working,
    /// Waiting for an answer, unknown to the daemon, or a status this client
    /// does not know, so a new one cannot break parsing.
    #[default]
    Other,
}

fn pane_status<'de, D: serde::Deserializer<'de>>(d: D) -> Result<PaneStatus, D::Error> {
    Ok(match Option::<String>::deserialize(d)?.as_deref() {
        Some("done") => PaneStatus::Done,
        Some("idle") => PaneStatus::Idle,
        Some("working") => PaneStatus::Working,
        _ => PaneStatus::Other,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgentSession {
    pub agent: String,
    pub kind: SessionKind,
    pub source: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionKind {
    Id,
    Path,
}

#[derive(Debug, Deserialize)]
pub struct ProcessInfoResult {
    pub process_info: ProcessInfo,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ProcessInfo {
    pub foreground_process_group_id: Option<u32>,
    pub shell_pid: Option<u32>,
    #[serde(default)]
    pub foreground_processes: Vec<Process>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Process {
    pub pid: u32,
    #[serde(default)]
    pub argv: Vec<String>,
    pub cwd: Option<String>,
}

impl ProcessInfo {
    /// Every process in the foreground job; none while the shell itself is in
    /// the foreground (an idle prompt).
    pub fn foreground_pids(&self) -> Vec<u32> {
        if self.foreground_process_group_id.is_none()
            || self.foreground_process_group_id == self.shell_pid
        {
            return Vec::new();
        }
        self.foreground_processes.iter().map(|p| p.pid).collect()
    }
}

#[cfg(test)]
mod tests;
