// ABOUTME: Builds workspace and agent navigation in the sidebar.
// ABOUTME: Shares sidebar presentation and accessibility behavior with the window.
//! The sidebar: spaces and agents, the rows that show them, and the hover
//! menu a resting pointer opens.
// i18n: migrated

mod accessibility;
mod activity;
mod agents;
mod cell;
mod device_drag;
mod host_row;
mod hover;
mod layout;
mod metrics;
mod note_line;
mod order;
pub(crate) mod paint;
mod rail;
mod render;
mod reorder;
mod row;
mod search;
pub(crate) mod slide;
mod sticky;
mod tag_groups;
mod tokens;
mod view;
mod wash;
mod workspaces;

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "integration-test"))]
pub(crate) mod layout_tests;

#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub(crate) mod native_tests;

pub(crate) use {
    activity::{Activity, ActivityPanel},
    agents::{Indicators, agent_name, pane_name, state_label, status_indicator, status_text},
    device_drag::DeviceDrag,
    hover::{HoverMenu, HoverRest},
    metrics::{HOST_ARROW_WIDTH, STATUS_WIDTH},
    paint::label_text,
    rail::SidebarMode,
    render::NestedReveal,
    reorder::WorkspaceDrag,
    search::SidebarSearch,
    view::SidebarView,
    workspaces::{workspace_entries, workspace_label},
};

pub(crate) use layout::SidebarMetrics;
pub(crate) use spaceport_sidebar::order::step as sidebar_step;

#[cfg(any(test, feature = "integration-test"))]
pub(crate) use metrics::LABEL_WIDTH;

pub(crate) use view::element as view_element;

use agents::agents_sort;
use metrics::*;
use row::RowBadge;
use workspaces::visible_workspace_entries;

pub(crate) const DEVICE_FOOTER_HEIGHT: f32 = 40.;

#[derive(Clone, Copy)]
pub(crate) enum SidebarDrag {
    Width { start: f32, width: f32 },
    Split,
}
