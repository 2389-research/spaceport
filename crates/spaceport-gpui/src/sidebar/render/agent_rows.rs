//! The agents list: every listed host's agents in the panel's one order, so a
//! blocked agent on one host can sit above an idle one on another.

use crate::{
    NavigationTarget, SpaceportWindow,
    sidebar::{
        AgentSection, Indicators, agent_name,
        agents::{agent_place, pane_name, state_label, status_text},
        cell::{AgentRow, Cell, RowContext, RowData, layout_for},
        layout::SidebarLook,
        line_height, scaled, tokens, wash,
    },
};
use gpui::{prelude::*, *};
use spaceport_client::protocol::{AgentStatus, ClientShellAgent, ClientShellSnapshot};

impl SpaceportWindow {
    /// Appends a row for each agent `panel_agents` lists, in that order, and
    /// returns the list with how many rows it painted and where the selected
    /// host's focused agent sits among them, for the reveal. A row's host
    /// shapes its own context, as the agent may belong to any listed host.
    pub(super) fn append_agent_rows(
        &self,
        mut list: Stateful<Div>,
        indicators: Indicators,
        look: SidebarLook,
        width: f32,
        show_sections: bool,
        cx: &mut Context<Self>,
    ) -> (Stateful<Div>, usize, Option<usize>) {
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let mut count = 0;
        let mut focused = None;
        let now = spaceport_client::agent_idle::now_ms();
        let mut previous = None;
        for (index, agent) in self.panel_agents_at(now) {
            let Some(endpoint) = self.endpoints.get(index) else {
                continue;
            };
            let host = endpoint.view();
            let selected = index == self.selected_endpoint;
            let live = self.live_of(index);
            let Some(snapshot) = live.snapshot.as_deref() else {
                continue;
            };
            let row_cx = RowContext {
                icon_only_agent: false,
                indicators,
                font,
                theme,
                look,
                width,
                // Agents list under their own heading, not under a host, but
                // each carries its host's colour.
                nest: 0.,
                mark: wash::HostMark::resolve(
                    &self.config.sidebar_style,
                    host.label,
                    selected,
                    theme,
                ),
                host: self.agent_host(&host),
                right_to_left: crate::direction::is_rtl(),
                painters: crate::sidebar::paint::PAINTERS,
            };
            let Some(lines) = self.panel_agent_lines(index, agent) else {
                continue;
            };
            if show_sections {
                let section = crate::sidebar::agents::sections::Section::of(
                    live,
                    agent,
                    now,
                    self.config.quiet_after,
                );
                if previous != Some(section) {
                    list = list.child(self.agent_section_heading(section, look, cx));
                    count += 1;
                    previous = Some(section);
                }
                if self.agent_section_folds.contains(&section) {
                    continue;
                }
            }
            if selected && agent.focused {
                focused = Some(count);
            }
            let gap = self.agent_row_gap(count);
            count += 1;
            list = list.child(self.agent_cell(
                index,
                agent,
                snapshot,
                &row_cx,
                lines,
                gap,
                false,
                previous.map(|section| (section, now)),
                cx,
            ));
        }
        (list, count, focused)
    }

    /// Whether the agents' rows follow a configured `[sidebar_layout.agents]`.
    fn custom_agent_rows(&self) -> bool {
        self.config.usage.inline
            && self.config.sidebar_layout.agents != crate::config::AgentLayout::default()
    }

    /// The host an agent's row names: a remote's label, among several hosts.
    fn agent_host<'a>(&self, host: &'a spaceport_endpoint::HostView<'a>) -> Option<&'a str> {
        (self.endpoints.len() > 1 && host.id != crate::endpoint::LOCAL).then_some(host.label)
    }

    /// The lines the expanded panel paints for host `index`'s agent, or None
    /// when it paints no row for it: [`Self::configured_agent_lines`] for
    /// that host. Stepping asks this too, so it skips what is not painted.
    pub(in crate::sidebar) fn panel_agent_lines(
        &self,
        index: usize,
        agent: &ClientShellAgent,
    ) -> Option<Vec<Vec<tokens::ResolvedToken>>> {
        let endpoint = self.endpoints.get(index)?;
        let host = endpoint.view();
        let snapshot = self.live_of(index).snapshot.as_deref()?;
        self.configured_agent_lines(agent, snapshot, self.agent_host(&host))
    }

    /// The lines a configured agent row shows, empty for Herdr's own row, or
    /// None when the configured rows leave this agent out.
    pub(super) fn configured_agent_lines(
        &self,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshot,
        host: Option<&str>,
    ) -> Option<Vec<Vec<tokens::ResolvedToken>>> {
        if !self.custom_agent_rows() {
            return Some(Vec::new());
        }
        tokens::agent_rows(
            &self.config.sidebar_layout.agents,
            agent,
            snapshot,
            host,
            status_text,
        )
    }

    /// Space above the agent row painted `count`-th in its list: a
    /// configured `row_gap` between configured rows, none otherwise.
    pub(super) fn agent_row_gap(&self, count: usize) -> f32 {
        if self.custom_agent_rows() && count > 0 {
            f32::from(self.config.sidebar_layout.agents.row_gap) * line_height(&self.config.sidebar)
        } else {
            0.
        }
    }

    /// One agent's row, for the agents list, under its device's header, or
    /// `nested` under its workspace, where the row above already names the
    /// place: the agent of endpoint `index`, drawn in `row_cx`, that shows
    /// its pane when clicked.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 121 lines")]
    pub(super) fn agent_cell(
        &self,
        index: usize,
        agent: &ClientShellAgent,
        snapshot: &ClientShellSnapshot,
        row_cx: &RowContext,
        mut lines: Vec<Vec<tokens::ResolvedToken>>,
        gap: f32,
        nested: bool,
        section_at: Option<(AgentSection, u64)>,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let rows = layout_for(self.config.layout.mode);
        let multi = self.endpoints.len() > 1;
        let selected = index == self.selected_endpoint;
        let focused = selected && agent.focused;
        let endpoint_id = self
            .endpoints
            .get(index)
            .map(|endpoint| endpoint.id.clone())
            .unwrap_or_default();
        let id = agent.pane_id.clone();
        let navigate_endpoint = endpoint_id.clone();
        let menu_endpoint = endpoint_id.clone();
        let menu_pane = id.clone();
        let menu_open = selected && self.pane_menu_target() == Some(id.as_str());
        let name = agent_name(agent);
        let pane = pane_name(agent, snapshot, name);
        let place = (!nested).then(|| agent_place(agent, snapshot)).flatten();
        let status = status_text(agent.agent_status);
        let state = state_label(agent, &status);
        let idle_age = section_at.and_then(|(_, now)| {
            if agent.agent_status != AgentStatus::Idle {
                return None;
            }
            let live = self.live_of(index);
            crate::sidebar::agents::sections::age_label(
                live.agent_idle(&id)?,
                live.agent_confirmed_at(now),
            )
        });
        let heading_names_state = matches!(section_at, Some((AgentSection::Idle(_), _)))
            && state.eq_ignore_ascii_case(&status);
        if heading_names_state {
            tokens::replace_idle_status(&mut lines, idle_age.as_deref());
        }
        let state = match idle_age.as_ref() {
            Some(age) => std::borrow::Cow::Owned(format!("{state} · {age}")),
            None => state,
        };
        let icon = crate::icons::AgentIcon::from_identity(agent.agent.as_deref());
        // An integration can supply a session name in place of its app name.
        // Keep that fallback, and names whose app has only a generic icon.
        let app_label = agent.display_agent.as_deref() == Some(name)
            || agent
                .agent
                .as_deref()
                .is_some_and(|id| id.eq_ignore_ascii_case(name));
        let mut row_cx = RowContext {
            icon_only_agent: app_label
                && icon != crate::icons::AgentIcon::Generic
                && row_cx.width - row_cx.nest <= scaled(row_cx.font, crate::sidebar::SIDEBAR_WIDTH),
            ..*row_cx
        };
        let visible_status = if heading_names_state {
            idle_age.map(std::borrow::Cow::Owned)
        } else {
            (idle_age.is_some()
                || self
                    .config
                    .sidebar_layout
                    .agents
                    .shows_status_text(agent.agent.as_deref()))
            .then(|| state.clone())
        };
        let (lines, spoken, status_text) = if lines.is_empty()
            && let Some(row) = crate::sidebar::agents::terminal_row::TerminalRow::from_snapshot(
                agent,
                snapshot,
                pane.as_deref(),
            ) {
            row_cx.icon_only_agent = true;
            let host = &self.endpoints[index].label;
            (
                row.lines(name, host, visible_status.as_deref()),
                row.spoken(name, host, &state),
                None,
            )
        } else {
            let spoken = crate::sidebar::accessibility::agent_name(
                name,
                pane.as_deref(),
                &state,
                place,
                row_cx.host,
            );
            (lines, spoken, visible_status)
        };
        Cell::new(
            rows,
            RowData::Agent(AgentRow {
                key: format!("agent-{id}"),
                name,
                icon,
                status: agent.agent_status,
                place,
                pane,
                status_text,
                lines,
            }),
            &row_cx,
        )
        .selected(focused)
        .highlighted(menu_open)
        .row()
        .when(gap > 0., |row| row.mt(px(gap)))
        .id(SharedString::from(format!("agent-{endpoint_id}-{id}")))
        .role(Role::ListItem)
        .aria_label(spoken)
        .aria_selected(focused)
        .map(|control| self.chrome_stop(crate::window::chrome_focus::Region::Sidebar, control))
        .when(multi, |row| {
            row.debug_selector(|| format!("agent-{endpoint_id}-{id}"))
        })
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                cx.stop_propagation();
                this.open_agent_menu(&menu_endpoint, &menu_pane, event.position, window, cx);
            }),
        )
        .on_click(cx.listener(move |this, _, window, cx| {
            this.navigate_endpoint(&navigate_endpoint, NavigationTarget::Pane(&id), cx);
            window.focus(&this.focus, cx);
        }))
    }
}
