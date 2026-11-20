// ABOUTME: Validates each profile workload before the existing sampler starts its clocks.
// ABOUTME: Keeps Core projections separate from TUI presented frames and capture qualification.
use super::{Observation, Result, Targets, ensure, foreground};

#[derive(Clone, Copy)]
pub(super) struct CoreTarget<'a> {
    pub(super) core_id: &'a str,
    pub(super) tab: u64,
    pub(super) terminal: &'a str,
}

#[derive(Clone, Copy)]
pub(super) enum Workload<'a> {
    TwoTui(&'a Targets),
    Mixed {
        tui: &'a Targets,
        core: CoreTarget<'a>,
    },
}

impl<'a> Workload<'a> {
    fn targets(self) -> &'a Targets {
        match self {
            Self::TwoTui(targets) | Self::Mixed { tui: targets, .. } => targets,
        }
    }

    pub(super) fn populated(self, observation: &Observation) -> bool {
        match self {
            Self::TwoTui(targets) => super::populated(observation, targets),
            Self::Mixed { .. } => false,
        }
    }

    pub(super) fn validate(self, observation: &Observation, working: bool) -> Result<()> {
        ensure!(
            foreground(observation),
            "refusing foreground qualification: inactive, hidden, occluded, or non-key window; {observation:?}"
        );
        match self {
            Self::TwoTui(_) => ensure!(
                self.populated(observation),
                "refusing sample without two populated current presented groups; {observation:?}"
            ),
            Self::Mixed { core, .. } => ensure!(
                self.populated(observation),
                "refusing mixed sample without the generated TUI and projected Core: core_id={} tab={} terminal={}; capture inspection remains required; {observation:?}",
                core.core_id,
                core.tab,
                core.terminal,
            ),
        }
        ensure!(
            observation
                .synthetic_working_panes
                .contains(&self.targets().pane)
                == working,
            "refusing sample with changed synthetic status: expected_working={working}; {observation:?}"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests;
