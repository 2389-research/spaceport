#![allow(clippy::unwrap_used)]

use spaceport_sidebar::agents::agent_labels;

use super::{
    STATUS_DOT_UNKNOWN, STATUS_WIDTH, agent_name,
    agents::{Indicators, agent_place, pane_name, status_indicator},
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
use spaceport_sidebar::agents::{PANE_CAPTION_LIMIT, status_style};

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
mod listing;
mod mark_sizes;
mod pane_names;
mod row_text;
mod statuses;
mod working_marks;
mod workspace_marks;
