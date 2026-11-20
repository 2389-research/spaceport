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
            Self::Mixed { tui, core } => {
                if observation.workspace.as_deref() != Some(&tui.workspace)
                    || observation.groups.len() != 2
                {
                    return false;
                }
                let Some(tui_group) = observation.groups.iter().find(|group| group.shown_terminal)
                else {
                    return false;
                };
                let Some(core_group) = observation
                    .groups
                    .iter()
                    .find(|group| !group.shown_terminal)
                else {
                    return false;
                };
                let has_area = |bounds: &[f32; 4]| {
                    bounds.iter().all(|value| value.is_finite()) && bounds[2] > 0. && bounds[3] > 0.
                };
                tui_group.core.is_none()
                    && tui_group.connected
                    && tui_group.tab.is_some()
                    && has_area(&tui_group.bounds)
                    && tui_group.presented.as_ref().is_some_and(|frame| {
                        frame.current
                            && frame.boot == tui.boot
                            && frame.columns > 0
                            && frame.rows > 0
                            && frame.pane_ids.contains(&tui.pane)
                            && frame.nonblank_cells > 0
                            && frame.marker_visible
                    })
                    && has_area(&core_group.bounds)
                    && core_group.core.as_ref().is_some_and(|observed| {
                        observed.core_id.as_deref() == Some(core.core_id)
                            && observed.tab == core.tab
                            && observed.terminal == core.terminal
                            && !observed.observe
                            && observed.connected
                            && !observed.ended
                            && !observed.failed
                            && observed.last_layout.as_ref().is_some_and(has_area)
                            && observed.projected.as_ref().is_some_and(|frame| {
                                frame.columns > 0
                                    && frame.rows > 0
                                    && frame.nonblank_cells > 0
                                    && frame.projected_marker
                            })
                    })
            }
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
