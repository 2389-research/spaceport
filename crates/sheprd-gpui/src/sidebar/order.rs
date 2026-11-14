//! The order the sidebar lists workspaces and agents in, which the previous,
//! next, and numbered shortcuts step through, as Herdr's TUI steps through
//! its own sidebar: every listed host's rows, top to bottom.

use super::{SidebarMode, agents::panel_agents, visible_workspace_entries};
use crate::{SheprdWindow, controls::Command, endpoint::Endpoint, state::LiveState};
use sheprd_client::protocol::ClientShellAgent;

/// A listed row: its host, by endpoint index, and the workspace or agent
/// pane it shows.
pub(crate) type Row<'a> = (usize, &'a str);

// What the sidebar lists. The expanded list and the rail paint through these
// and the shortcuts step through them, so a step lands on a painted row.
impl SheprdWindow {
    /// Each host the sidebar lists, in its order, with its current state.
    pub(super) fn listed_hosts(&self) -> impl Iterator<Item = (usize, &Endpoint, &LiveState)> {
        self.displayed_devices()
            .into_iter()
            .filter(|(_, endpoint)| self.device_visible(&endpoint.id))
            .map(|(index, endpoint)| (index, endpoint, self.live_of(index)))
    }

    /// Whether the expanded list arranges its hosts under tag sections, which
    /// fold. `headed` is whether hosts have headers at all, and `query` what
    /// the device tree is searched for: results are not arranged by group.
    pub(super) fn lists_tag_sections(&self, headed: bool, query: &str) -> bool {
        headed && query.is_empty() && self.device_tags.any()
    }

    /// Whether `endpoint`'s own arrow folds its rows away. One host alone
    /// has no header to fold.
    pub(super) fn host_folded(&self, endpoint: &Endpoint) -> bool {
        self.endpoints.len() > 1 && endpoint.collapsed
    }

    /// The workspace rows the current mode paints, in its order. The rail
    /// has no tag sections, so only the expanded list hides a folded
    /// section's hosts. A hidden sidebar steps as the expanded one. The
    /// Devices layout and a lifted device paint headers only; stepping keeps
    /// the workspaces under them.
    pub(crate) fn sidebar_workspaces(&self) -> Vec<Row<'_>> {
        let sections = self.sidebar_mode() != SidebarMode::Rail
            && self.lists_tag_sections(self.endpoints.len() > 1, "");
        self.listed_hosts()
            .filter(|(_, endpoint, _)| {
                !(sections && self.device_tag_folded(&endpoint.id)) && !self.host_folded(endpoint)
            })
            .filter_map(|(index, endpoint, live)| {
                Some((index, endpoint, live.snapshot.as_deref()?))
            })
            .flat_map(|(index, endpoint, snapshot)| {
                visible_workspace_entries(&snapshot.workspaces, &endpoint.collapsed_repos)
                    .into_iter()
                    .map(move |(row, _, _)| (index, snapshot.workspaces[row].workspace_id.as_str()))
            })
            .collect()
    }

    /// Every listed host's agents in the panel's order, by host index. The
    /// expanded list and the rail each paint these less the rows their own
    /// rule leaves out.
    pub(super) fn panel_agents(&self) -> Vec<(usize, &ClientShellAgent)> {
        panel_agents(
            self.listed_hosts()
                .filter(|(_, endpoint, _)| endpoint.enabled)
                .map(|(index, _, live)| (index, live)),
            self.agent_sort,
        )
    }

    /// The agent rows the current mode paints, in its order, whether or not
    /// the agents are shown, as Herdr steps through its agents either way.
    /// The rail folds a collapsed host's agents away; the expanded list
    /// leaves out an agent its configured rows have no lines for. A hidden
    /// sidebar steps as the expanded one.
    pub(crate) fn sidebar_agents(&self) -> Vec<Row<'_>> {
        let rail = self.sidebar_mode() == SidebarMode::Rail;
        self.panel_agents()
            .into_iter()
            .filter(|&(index, agent)| {
                if rail {
                    !self.host_folded(&self.endpoints[index])
                } else {
                    self.panel_agent_lines(index, agent).is_some()
                }
            })
            .map(|(index, agent)| (index, agent.pane_id.as_str()))
            .collect()
    }
}

/// Which row a sidebar step lands on. `current` is the focused row, if the
/// sidebar lists it; `selected` is the selected host. Previous and next wrap
/// around, and start from an end when nothing listed is focused. A numbered
/// workspace counts only the selected host's rows and a numbered agent every
/// host's, as Herdr numbers them.
pub(crate) fn step(
    rows: &[Row<'_>],
    current: Option<usize>,
    selected: usize,
    command: Command,
) -> Option<usize> {
    let len = rows.len();
    if len == 0 {
        return None;
    }
    match command {
        Command::PreviousWorkspace | Command::PreviousAgent => {
            Some(current.map_or(len - 1, |index| (index + len - 1) % len))
        }
        Command::NextWorkspace | Command::NextAgent => {
            Some(current.map_or(0, |index| (index + 1) % len))
        }
        Command::WorkspaceNumber(number) => rows
            .iter()
            .enumerate()
            .filter(|(_, (host, _))| *host == selected)
            .nth(usize::from(number).checked_sub(1)?)
            .map(|(index, _)| index),
        Command::AgentNumber(number) => {
            let index = usize::from(number).checked_sub(1)?;
            (index < len).then_some(index)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_wrap_and_start_from_an_end() {
        let rows = [(0, "a"), (0, "b"), (1, "c")];
        for (command, current, expected) in [
            (Command::NextWorkspace, Some(0), Some(1)),
            (Command::NextWorkspace, Some(2), Some(0)),
            (Command::NextWorkspace, None, Some(0)),
            (Command::PreviousWorkspace, Some(0), Some(2)),
            (Command::PreviousWorkspace, Some(2), Some(1)),
            (Command::PreviousWorkspace, None, Some(2)),
            (Command::NextAgent, Some(1), Some(2)),
            (Command::PreviousAgent, None, Some(2)),
        ] {
            assert_eq!(step(&rows, current, 0, command), expected, "{command:?}");
        }
        for command in [Command::NextWorkspace, Command::PreviousAgent] {
            assert_eq!(step(&[], None, 0, command), None);
        }
        assert_eq!(step(&rows, Some(0), 0, Command::Tab), None);
    }

    #[test]
    fn numbers_count_from_one() {
        let rows = [(0, "a"), (1, "b"), (0, "c"), (1, "d")];
        // A numbered workspace counts the selected host's rows only.
        assert_eq!(step(&rows, None, 0, Command::WorkspaceNumber(2)), Some(2));
        assert_eq!(step(&rows, None, 1, Command::WorkspaceNumber(1)), Some(1));
        assert_eq!(step(&rows, None, 1, Command::WorkspaceNumber(3)), None);
        assert_eq!(step(&rows, None, 0, Command::WorkspaceNumber(0)), None);
        // A numbered agent counts every host's.
        assert_eq!(step(&rows, None, 0, Command::AgentNumber(4)), Some(3));
        assert_eq!(step(&rows, None, 0, Command::AgentNumber(5)), None);
        assert_eq!(step(&rows, None, 0, Command::AgentNumber(0)), None);
    }
}
