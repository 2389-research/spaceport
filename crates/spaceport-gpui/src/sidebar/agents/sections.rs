// ABOUTME: Groups the Agents panel using idle spells confirmed by each connection.
// ABOUTME: Keeps daemon-local sequences inside their endpoint when ordering undated rows.
use crate::{i18n::fl, state::LiveState};
use spaceport_client::{
    agent_idle::{IdleAgeGroup, IdleObservation},
    protocol::{AgentStatus, ClientShellAgent},
};
use std::cmp::Reverse;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Section {
    Waiting,
    Working,
    Done,
    Idle(IdleAgeGroup),
    Quiet,
    Other,
    Offline,
}

impl Section {
    pub(in crate::sidebar) fn of(
        live: &LiveState,
        agent: &ClientShellAgent,
        now: u64,
        quiet_after: spaceport_settings::QuietAfter,
    ) -> Self {
        if !live.status.is_connected() {
            return Self::Offline;
        }
        match agent.agent_status {
            AgentStatus::Blocked => Self::Waiting,
            AgentStatus::Working => Self::Working,
            AgentStatus::Done => Self::Done,
            AgentStatus::Idle => match live.agent_idle(&agent.pane_id) {
                Some(idle)
                    if now.saturating_sub(idle.first_observed_ms) >= quiet_after.milliseconds() =>
                {
                    Self::Quiet
                }
                Some(idle) => Self::Idle(idle.group(now)),
                None => Self::Idle(IdleAgeGroup::Unplaced),
            },
            AgentStatus::Unknown => Self::Other,
        }
    }

    pub(in crate::sidebar) fn heading(self) -> (&'static str, String) {
        match self {
            Self::Waiting => ("agents-waiting", fl!("sidebar-agents-waiting")),
            Self::Working => ("agents-working", fl!("sidebar-agents-working")),
            Self::Done => ("agents-done", fl!("sidebar-agents-done")),
            Self::Idle(IdleAgeGroup::Recent) => {
                ("agents-recent", fl!("sidebar-agents-idle-recent"))
            }
            Self::Idle(IdleAgeGroup::OneToFiveHours) => {
                ("agents-hours", fl!("sidebar-agents-idle-hours"))
            }
            Self::Idle(IdleAgeGroup::FiveHoursOrMore) => {
                ("agents-long-idle", fl!("sidebar-agents-idle-long"))
            }
            Self::Idle(IdleAgeGroup::Unplaced) => ("agents-idle", fl!("sidebar-agents-idle")),
            Self::Quiet => ("agents-quiet", fl!("sidebar-agents-quiet")),
            Self::Other => ("agents-other", fl!("sidebar-agents-other")),
            Self::Offline => ("agents-offline", fl!("sidebar-agents-offline")),
        }
    }
}

pub(super) fn order(
    index: usize,
    live: &LiveState,
    agent: &ClientShellAgent,
    now: u64,
    quiet_after: spaceport_settings::QuietAfter,
) -> (Section, Reverse<u64>, usize, Reverse<u64>) {
    let section = Section::of(live, agent, now, quiet_after);
    let recency = match section {
        Section::Idle(IdleAgeGroup::Unplaced) => 0,
        Section::Idle(_) | Section::Quiet => live
            .agent_idle(&agent.pane_id)
            .map_or(0, |idle| idle.first_observed_ms),
        _ => live.agent_recency.of(&agent.pane_id),
    };
    (
        section,
        Reverse(recency),
        index,
        Reverse(agent.state_change_seq),
    )
}

pub(in crate::sidebar) fn age_label(idle: IdleObservation, now: u64) -> Option<String> {
    if idle.group(now) == IdleAgeGroup::Unplaced {
        return None;
    }
    let minutes = now.saturating_sub(idle.first_observed_ms) / 60_000;
    let amount = if minutes >= 60 {
        fl!("sidebar-idle-age-hours", count = (minutes / 60).to_string())
    } else {
        fl!("sidebar-idle-age-minutes", count = minutes.to_string())
    };
    Some(if idle.onset_observed {
        amount
    } else {
        fl!("sidebar-idle-at-least", age = amount)
    })
}
