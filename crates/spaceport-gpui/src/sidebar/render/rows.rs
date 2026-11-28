//! The sidebar's rows, built from measured state: the spaces list with its tag
//! sections, host headers and workspace rows, and the agents panel.

use super::measure::SidebarMeasure;
use crate::{
    SpaceportWindow,
    i18n::fl,
    sidebar::{MoveBlock, agents::Indicators, cell::RowContext, label_text, wash},
};
use gpui::{prelude::*, *};

mod workspaces;

use workspaces::HostRows;

/// The two lists a frame paints, and the positions the stages after this one
/// pin, reveal, and drag against.
pub(super) struct SidebarRows {
    /// The spaces list: tag sections, host headers, then each host's rows.
    pub(super) spaces: Stateful<Div>,
    /// The agents panel's list, without rows when the layout nests agents.
    pub(super) agents: Stateful<Div>,
    /// Search results, which stand in for the spaces list.
    pub(super) search: Option<Stateful<Div>>,
    /// Child positions of the highlighted workspace row and agent row, for the
    /// one-time reveal.
    pub(super) highlighted: [Option<usize>; 2],
    /// A plugin view hid every agent, rather than there being none.
    pub(super) filtered: bool,
    /// The focused agent's row when agents nest under their workspaces.
    pub(super) nested_focus: Option<SharedString>,
    /// Each host header's endpoint index and child position in the spaces
    /// list, for the header pinned over it.
    pub(super) host_rows: Vec<(usize, usize)>,
    /// A lifted workspace row: each drop unit's rows in the spaces list, and
    /// the move every gap makes, for the pointer handler.
    pub(super) drop_rows: Vec<(usize, usize, Pixels)>,
    pub(super) drop_requests: Vec<Option<MoveBlock>>,
    pub(super) drop_dragged: usize,
    /// Whether a row is mid-slide, so the frame asks for another.
    pub(super) sliding: bool,
}

/// The spaces list as it is built, with the child positions rows record as
/// they are appended.
pub(super) struct SpacesRows {
    pub(super) list: Stateful<Div>,
    /// Child positions appended so far.
    pub(super) rows: usize,
    /// The child position of the selected host's focused workspace row.
    pub(super) highlighted: Option<usize>,
    pub(super) host_rows: Vec<(usize, usize)>,
    pub(super) drop_rows: Vec<(usize, usize, Pixels)>,
    pub(super) drop_requests: Vec<Option<MoveBlock>>,
    pub(super) drop_dragged: usize,
    pub(super) sliding: bool,
}

impl SpacesRows {
    fn new(list: Stateful<Div>, sliding: bool) -> Self {
        Self {
            list,
            rows: 0,
            highlighted: None,
            host_rows: Vec::new(),
            drop_rows: Vec::new(),
            drop_requests: Vec::new(),
            drop_dragged: 0,
            sliding,
        }
    }
}

impl SpaceportWindow {
    /// Builds the frame's rows: each listed host's header and workspace rows,
    /// the devices layout's agent rows in their place, and the agents panel.
    #[allow(clippy::too_many_lines, reason = "2026-11-27 baseline: 168 lines")]
    pub(super) fn sidebar_rows(
        &self,
        m: &SidebarMeasure<'_>,
        indicators: Indicators,
        cx: &mut Context<Self>,
    ) -> SidebarRows {
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let search = (m.searchable && self.sidebar_search.query().is_some()).then(|| {
            self.sidebar_search_results(self.sidebar_search_hits(cx), indicators, m.look, cx)
        });
        let mut spaces = SpacesRows::new(
            div()
                .id("spaces-scroll")
                .debug_selector(|| "spaces-scroll".into())
                .role(Role::List)
                .aria_label(if m.devices {
                    fl!("sidebar-devices-label")
                } else {
                    fl!("sidebar-workspaces-label")
                })
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(&self.sidebar_scroll[0]),
            m.device_plan.as_ref().is_some_and(|plan| plan.sliding),
        );
        let mut agents = div()
            .id("agents-scroll")
            .debug_selector(|| "agents-scroll".into())
            .role(Role::List)
            .aria_label(fl!("sidebar-agents-label"))
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.sidebar_scroll[1]);
        let tag_filter = self.device_tag_filter.as_deref();
        let mut section = None;
        // The focused agent's row when agents nest under their workspaces.
        let mut nested_focus = None;
        // A plugin view hid every agent, rather than there being none.
        let filtered = self.config.show_agents && self.filtered_agent_view();
        for (endpoint_index, endpoint, live) in
            self.listed_hosts().take_while(|_| !m.workspaces_collapsed)
        {
            if m.devices && !self.device_listed(endpoint_index, &m.query) {
                continue;
            }
            let view = endpoint.view();
            if m.grouped {
                let own = self.device_tags.section(view.id, tag_filter);
                if !section.is_some_and(|last| super::super::tag_groups::same_section(last, own)) {
                    section = Some(own);
                    let count = self.tag_section_count(own);
                    spaces.list =
                        spaces
                            .list
                            .child(self.tag_section(own, count, m.heading_look, cx));
                    spaces.rows += 1;
                }
                if self.device_tag_folded(view.id) {
                    continue;
                }
            }
            let selected = endpoint_index == self.selected_endpoint;
            let mark =
                wash::HostMark::resolve(&self.config.sidebar_style, view.label, selected, theme);
            if m.headed {
                spaces.host_rows.push((endpoint_index, spaces.rows));
                let header = self.host_row(endpoint, selected, m.look, m.width, false, cx);
                spaces.list = self.place_host_row(
                    spaces.list,
                    header,
                    (endpoint_index, spaces.rows),
                    m.device_plan.as_ref(),
                    cx,
                );
                spaces.rows += 1;
            }
            if m.device_plan.is_some() {
                continue;
            }
            if m.devices {
                let (list, rows, focused) = self.append_device_agents(
                    spaces.list,
                    endpoint_index,
                    indicators,
                    m.look,
                    m.width,
                    cx,
                );
                spaces.list = list;
                if let Some(row) = focused {
                    spaces.highlighted = Some(spaces.rows + row);
                }
                spaces.rows += rows;
                continue;
            }
            let row_cx = RowContext {
                icon_only_agent: false,
                indicators,
                font,
                theme,
                look: m.look,
                width: m.width,
                nest: if m.multi { m.look.nest_indent() } else { 0. },
                mark,
                host: (m.multi && view.id != crate::endpoint::LOCAL).then_some(view.label),
                right_to_left: crate::direction::is_rtl(),
                painters: crate::sidebar::paint::PAINTERS,
            };
            let Some(snapshot) = &live.snapshot else {
                continue;
            };
            let (next, focused) = self.append_workspace_rows(
                spaces,
                HostRows {
                    view,
                    index: endpoint_index,
                    selected,
                    snapshot,
                    mutes: crate::notification_mutes::Daemon::of(endpoint),
                    ports: crate::listening_ports::Daemon::of(&endpoint.connection.target),
                },
                &row_cx,
                m,
                cx,
            );
            spaces = next;
            nested_focus = nested_focus.or(focused);
        }
        if self.core_enabled() && !m.devices && !m.workspaces_collapsed && search.is_none() {
            spaces.list = spaces.list.child(self.core_workspace_rows(cx));
        }
        let mut agent_count = 0;
        let mut agent_highlighted = None;
        if m.agents_panel {
            (agents, agent_count, agent_highlighted) = self.append_agent_rows(
                agents,
                indicators,
                m.look,
                m.width,
                self.shows_agent_sections(filtered),
                cx,
            );
        }
        if agent_count == 0 {
            agents = agents.child(
                div()
                    .px(px(m.content_x))
                    .text_color(rgb(theme.muted))
                    .truncate()
                    .child(label_text(&if filtered {
                        fl!("sidebar-no-matching-agents")
                    } else {
                        fl!("sidebar-no-agents")
                    })),
            );
        }
        SidebarRows {
            spaces: spaces.list,
            agents,
            search,
            highlighted: [spaces.highlighted, agent_highlighted],
            filtered,
            nested_focus,
            host_rows: spaces.host_rows,
            drop_rows: spaces.drop_rows,
            drop_requests: spaces.drop_requests,
            drop_dragged: spaces.drop_dragged,
            sliding: spaces.sliding,
        }
    }
}
