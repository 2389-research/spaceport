//! The sidebar's look and the choices that pick its sections, read once before
//! any row is built.

use crate::{SpaceportWindow, config::AgentsPlacement};
use gpui::{prelude::*, *};
use std::time::Instant;

use super::super::{
    DevicePlan, HOST_ARROW_WIDTH,
    cell::{RowLayout, layout_for},
    layout::{self, SidebarLook},
    scaled, sidebar_width, sticky,
};

/// What a sidebar frame's look and choices come to. Later stages read it
/// instead of the config, so every row and heading in the frame agrees on the
/// width, the density, and which sections exist at all.
pub(super) struct SidebarMeasure<'a> {
    /// The panel's width: its setting, clamped to this viewport.
    pub(super) width: f32,
    /// The spaces section's share of the panel's height.
    pub(super) split: f32,
    /// The metrics rows are drawn with, stepped in under a tag's heading.
    pub(super) look: SidebarLook,
    /// The look headings keep, without the step-in rows take under one.
    pub(super) heading_look: SidebarLook,
    /// Where row content starts.
    pub(super) content_x: f32,
    /// The row layout this frame's mode selects.
    pub(super) row_layout: &'static dyn RowLayout,
    /// The workspace row a menu was opened for, which keeps looking hovered
    /// while the pointer is over the menu.
    pub(super) menu_target: Option<&'a str>,
    /// The spaces layout draws more than a label, with a gap between rows.
    pub(super) spaces_custom: bool,
    /// The Devices layout heads every device and lists its agents under it.
    pub(super) devices: bool,
    /// The spaces list stands in for by its search results.
    pub(super) searchable: bool,
    pub(super) workspaces_collapsed: bool,
    /// The device tree narrows in place by its own field.
    pub(super) tree_search: bool,
    /// More than one host is listed, so rows carry their host.
    pub(super) multi: bool,
    /// Hosts are headed, by being several or by the Devices layout.
    pub(super) headed: bool,
    /// Agents have their own section below the spaces list.
    pub(super) agents_panel: bool,
    /// Agents nest under their workspaces instead.
    pub(super) nested_agents: bool,
    /// The device tree's filter; empty unless the tree's field is shown.
    pub(super) query: String,
    /// Tag sections group the rows rather than search results or a fold.
    pub(super) grouped: bool,
    /// The folded list's headers while a device is lifted.
    pub(super) device_plan: Option<DevicePlan>,
    /// The frame's clock, so every slide in it measures from the same now.
    pub(super) now: Instant,
}

impl SpaceportWindow {
    /// Reads this frame's width, look, and section choices, and settles each
    /// list's scroll offset to the extent the last frame measured. The search
    /// field takes the heading's face here, as the heading it stands in for.
    pub(super) fn sidebar_measure(
        &self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> SidebarMeasure<'_> {
        let width = sidebar_width(self.sidebar_width, f32::from(window.viewport_size().width));
        let split = self.sidebar_split.unwrap_or(0.5).clamp(0.1, 0.9);
        let heading_look = layout::for_config(&self.config, self.fill().mark());
        let font = &self.config.sidebar;
        let theme = &self.theme;
        let menu_target = self.workspace_menu_target();
        let spaces_custom = self.config.usage.inline
            && self.config.sidebar_layout.spaces != crate::config::SpaceLayout::default();
        // The field is the heading, so it takes the heading's face.
        self.sidebar_search.input.update(cx, |input, cx| {
            input.set_appearance(font.clone(), theme.clone(), cx);
        });
        // Pinning and visibility below judge offsets, and the wheel sets them
        // without bounds.
        self.sidebar_scroll.iter().for_each(sticky::settle);
        // The Devices layout heads every device, even a lone one, and lists
        // its agents under it in place of its workspaces. Its own field
        // narrows that tree in place, so the sidebar's search, whose results
        // replace the list, gives way to it; one setting hides either.
        let devices = !self.config.layout.mode.lists_workspaces();
        // A hidden field keeps whatever it held, but never filters the list.
        let workspaces_collapsed = !devices && !self.sidebar_workspaces_expanded;
        let searchable = self.config.show_sidebar_search && !devices && !workspaces_collapsed;
        let tree_search = self.config.show_sidebar_search && devices;
        let multi = self.endpoints.len() > 1;
        let headed = multi || devices;
        let placement = self
            .config
            .show_agents
            .then_some(self.config.agents_placement);
        let agents_panel = placement == Some(AgentsPlacement::Panel) && !devices;
        let nested_agents = placement == Some(AgentsPlacement::Nested) && !devices;
        let query = if tree_search {
            self.device_tree_query(cx)
        } else {
            String::new()
        };
        // A lifted device folds the list to its headers.
        let device_plan = headed.then(|| self.device_plan(devices, &query)).flatten();
        // A lifted device folds the list to headers alone, and search results
        // are not arranged by group.
        let grouped = device_plan.is_none() && self.lists_tag_sections(headed, &query);
        // Sections are headings, so what lists under them steps in, with each
        // host's arrow below its section's dot.
        let look = SidebarLook {
            group_indent: if grouped {
                scaled(font, HOST_ARROW_WIDTH) + heading_look.density.host_gap()
            } else {
                0.
            },
            ..heading_look
        };
        SidebarMeasure {
            width,
            split,
            look,
            heading_look,
            content_x: heading_look.content_x(),
            row_layout: layout_for(self.config.layout.mode),
            menu_target,
            spaces_custom,
            devices,
            searchable,
            workspaces_collapsed,
            tree_search,
            multi,
            headed,
            agents_panel,
            nested_agents,
            query,
            grouped,
            device_plan,
            now: crate::motion::now(),
        }
    }
}
