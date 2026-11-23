#![allow(clippy::unwrap_used)]

use super::{
    STATUS_DOT_UNKNOWN, STATUS_WIDTH, agent_name,
    agents::{Indicators, agent_labels, agent_place, pane_name, status_indicator, status_style},
    layout_tests,
    render::header::header,
    workspace_label,
    workspaces::workspace_entries,
};
use crate::{
    config::{FontConfig, StatusIcons, Theme},
    contrast::Contrast,
};
use spaceport_client::protocol::{
    AgentStatus, ClientShellAgent, ClientShellPane, ClientShellSnapshot, ClientShellWorkspace,
};
use spaceport_sidebar::agents::PANE_CAPTION_LIMIT;

/// Every status, most urgent first.
const STATUSES: [AgentStatus; 5] = [
    AgentStatus::Working,
    AgentStatus::Blocked,
    AgentStatus::Done,
    AgentStatus::Idle,
    AgentStatus::Unknown,
];

mod agent_activity;
mod agent_order;
mod hierarchy;
mod host_mark;
mod listing;
mod mark_sizes;
mod pane_names;
mod row_text;
mod statuses;
mod working_marks;
mod workspace_marks;
