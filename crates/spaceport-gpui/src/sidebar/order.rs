//! The order the sidebar lists workspaces and agents in, which the previous,
//! next, and numbered shortcuts step through, as Herdr's TUI steps through
//! its own sidebar: every listed host's rows, top to bottom. What a step lands
//! on is `spaceport-sidebar`'s; which rows the sidebar lists is the window's.

use super::{SidebarMode, agents::panel_agents, visible_workspace_entries};
use crate::{SpaceportWindow, endpoint::Endpoint, state::LiveState};
use spaceport_client::protocol::ClientShellAgent;

pub(crate) use spaceport_sidebar::order::Row;

// What the sidebar lists. The expanded list and the rail paint through these
// and the shortcuts step through them, so a step lands on a painted row.
impl SpaceportWindow {
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

    pub(crate) fn has_live_idle_rows(&self) -> bool {
        self.config.show_agents
            && self.agent_sort == crate::preferences::AgentSort::Priority
            && self.listed_hosts().any(|(_, endpoint, live)| {
                endpoint.enabled()
                    && live.status.is_connected()
                    && live.snapshot.as_deref().is_some_and(|s| {
                        s.agent_view_label.is_none()
                            && s.agents
                                .iter()
                                .any(|a| live.agent_idle(&a.pane_id).is_some())
                    })
            })
    }

    /// Every listed host's agents in the panel's order, by host index. The
    /// expanded list and the rail each paint these less the rows their own
    /// rule leaves out.
    pub(super) fn panel_agents(&self) -> Vec<(usize, &ClientShellAgent)> {
        self.panel_agents_at(spaceport_client::agent_idle::now_ms())
    }

    pub(super) fn panel_agents_at(&self, now: u64) -> Vec<(usize, &ClientShellAgent)> {
        panel_agents(
            self.listed_hosts()
                .filter(|(_, endpoint, _)| endpoint.enabled())
                .map(|(index, _, live)| (index, live)),
            self.agent_sort,
            now,
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
