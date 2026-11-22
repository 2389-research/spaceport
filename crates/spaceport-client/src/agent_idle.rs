// ABOUTME: Tracks the age of an idle spell from authoritative Herdr snapshots.
// ABOUTME: Distinguishes a witnessed transition from an agent already idle on arrival.
use crate::protocol::{AgentStatus, ClientShellAgent, ClientShellSnapshot};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdleAgeGroup {
    Recent,
    OneToFiveHours,
    FiveHoursOrMore,
    Unplaced,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdleObservation {
    pub first_observed_ms: u64,
    pub onset_observed: bool,
}

impl IdleObservation {
    /// Use the last confirmation for cached rows and the current time for live ones.
    pub fn group(self, confirmed_at_ms: u64) -> IdleAgeGroup {
        match confirmed_at_ms.saturating_sub(self.first_observed_ms) {
            18_000_000.. => IdleAgeGroup::FiveHoursOrMore,
            3_600_000.. => IdleAgeGroup::OneToFiveHours,
            _ if self.onset_observed => IdleAgeGroup::Recent,
            _ => IdleAgeGroup::Unplaced,
        }
    }
}

pub fn now_ms() -> u64 {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}

#[derive(Clone, Debug)]
struct Agent {
    name: String,
    kind: Option<String>,
    status: AgentStatus,
    sequence: u64,
    revision: u64,
    idle: Option<IdleObservation>,
}

impl Agent {
    fn observe(&mut self, next: &ClientShellAgent, revision: u64, now_ms: u64, continuous: bool) {
        let name = crate::text::agent_name(next);
        let same_agent = self.name == name && self.kind == next.agent;
        if !matches!(next.agent_status, AgentStatus::Idle | AgentStatus::Done) {
            self.idle = None;
        } else if !same_agent
            || self.idle.is_none()
            || self.sequence != next.state_change_seq
            || (!continuous && next.state_change_seq == 0)
        {
            self.idle = Some(IdleObservation {
                first_observed_ms: now_ms,
                onset_observed: continuous
                    && same_agent
                    && matches!(self.status, AgentStatus::Working | AgentStatus::Blocked),
            });
        }
        if !same_agent {
            name.clone_into(&mut self.name);
            self.kind.clone_from(&next.agent);
        }
        self.sequence = next.state_change_seq;
        self.status = next.agent_status;
        self.revision = revision;
    }
}

#[derive(Clone, Debug, Default)]
pub struct IdleTracker {
    boot: String,
    revision: Option<u64>,
    continuous: bool,
    agents: Arc<HashMap<String, Agent>>,
}

impl IdleTracker {
    pub fn observe(&mut self, snapshot: &ClientShellSnapshot, now_ms: u64) {
        if self.boot != snapshot.boot_id {
            self.boot.clone_from(&snapshot.boot_id);
            self.revision = None;
            Arc::make_mut(&mut self.agents).clear();
            self.continuous = false;
        }
        if self.revision.is_some_and(|last| last > snapshot.revision) {
            return;
        }
        if self.revision == Some(snapshot.revision) && self.continuous {
            return;
        }
        let agents = Arc::make_mut(&mut self.agents);
        for next in &snapshot.agents {
            let previous = if let Some(previous) = agents.get_mut(&next.pane_id) {
                previous
            } else {
                agents.entry(next.pane_id.clone()).or_insert_with(|| Agent {
                    name: String::new(),
                    kind: None,
                    status: AgentStatus::Unknown,
                    sequence: 0,
                    revision: snapshot.revision,
                    idle: None,
                })
            };
            previous.observe(next, snapshot.revision, now_ms, self.continuous);
        }
        agents.retain(|_, agent| agent.revision == snapshot.revision);
        self.revision = Some(snapshot.revision);
        self.continuous = true;
    }

    /// Preserve unchanged sequence evidence while ruling out a witnessed transition across the gap.
    pub fn interrupt(&mut self) {
        self.continuous = false;
    }

    pub fn get(&self, pane_id: &str) -> Option<IdleObservation> {
        self.agents.get(pane_id)?.idle
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;
