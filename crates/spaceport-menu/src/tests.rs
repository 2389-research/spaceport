#![allow(clippy::unwrap_used)]
//! The payloads, fences and rows the moved models answer with, over one
//! snapshot: two checkouts of one repository, one workspace that stands in its
//! own repository, and two panes in one tab.

use serde_json::json;
use spaceport_client::protocol::ClientShellSnapshot;

mod appear;
mod chrome;
mod group_rows;
mod keybinds;
mod left_checkout;
mod page_set;
mod page_title;
mod pane_payloads;
mod picker_lines;
mod tab_payloads;
mod workspace_targets;

/// The repository the snapshot's linked checkouts hang off, in the shape
/// Herdr reports a main checkout's Git directory.
pub(crate) const REPO_KEY: &str = "/fixture/agent-launcher/.git";

/// The endpoint snapshot every test here works from.
pub(crate) fn snapshot() -> ClientShellSnapshot {
    serde_json::from_value(json!({
        "boot_id": "menu-test", "revision": 1,
        "config_diagnostic": null, "product_announcement": null, "update_available": null,
        "update_install_command": "", "server_keybindings_toml": null,
        "latest_release_notes_available": false, "integration_updates_available": false,
        "worktree_directory": "", "release_notes": null, "agent_view_label": null,
        "focused_workspace_id": "w1", "focused_tab_id": "t1", "focused_pane_id": "p2",
        "tab_bar_right": [], "tab_bar_right_separator": "", "agent_order": [],
        "agents": [], "commands": [],
        "workspaces": [
            {
                "workspace_id": "w0", "active_tab_id": "t0", "new_workspace_cwd": "/tmp",
                "number": 1, "label": "herdr", "custom_label": false, "branch": "main",
                "worktree": null, "tokens": [], "focused": false, "agent_status": "working"
            },
            {
                "workspace_id": "w1", "active_tab_id": "t1", "new_workspace_cwd":
                REPO_KEY.trim_end_matches("/.git"),
                "number": 2, "label": "agent-launcher", "custom_label": false,
                "branch": "develop",
                "worktree": {"key": REPO_KEY, "label": "agent-launcher", "is_linked_worktree": false},
                "tokens": [], "focused": true, "agent_status": "working"
            },
            {
                "workspace_id": "w2", "active_tab_id": "t2", "new_workspace_cwd": "/tmp",
                "number": 3, "label": "agent-launcher", "custom_label": false,
                "branch": "worktree/child",
                "worktree": {"key": REPO_KEY, "label": "agent-launcher", "is_linked_worktree": true},
                "tokens": [], "focused": false, "agent_status": "working"
            },
            {
                "workspace_id": "w3", "active_tab_id": "t3", "new_workspace_cwd": "/srv/elsewhere",
                "number": 4, "label": "elsewhere", "custom_label": false, "branch": null,
                "worktree": {
                    "key": "/srv/elsewhere/.git", "label": "elsewhere", "is_linked_worktree": false
                },
                "tokens": [], "focused": false, "agent_status": "working"
            }
        ],
        "tabs": [
            {
                "tab_id": "t0", "workspace_id": "w0", "number": 1, "label": "main",
                "custom_label": false, "zoomed": false, "focused": false,
                "agent_status": "working"
            },
            {
                "tab_id": "t1", "workspace_id": "w1", "number": 1, "label": "main",
                "custom_label": false, "zoomed": false, "focused": true,
                "agent_status": "working"
            },
            {
                "tab_id": "t2", "workspace_id": "w2", "number": 1, "label": "child",
                "custom_label": false, "zoomed": false, "focused": false,
                "agent_status": "working"
            },
            {
                "tab_id": "t3", "workspace_id": "w3", "number": 1, "label": "shell",
                "custom_label": false, "zoomed": false, "focused": false,
                "agent_status": "working"
            }
        ],
        "panes": [
            {
                "pane_id": "p0", "workspace_id": "w0", "tab_id": "t0", "label": "shell",
                "cwd": "/tmp", "foreground_cwd": "/tmp", "focused": false,
                "right_click_passthrough": false
            },
            {
                "pane_id": "p1", "workspace_id": "w1", "tab_id": "t1", "label": "shell",
                "cwd": "/srv", "foreground_cwd": "/srv", "focused": false,
                "right_click_passthrough": false
            },
            {
                "pane_id": "p2", "workspace_id": "w1", "tab_id": "t1", "label": "build",
                "cwd": "/srv", "foreground_cwd": "/srv", "focused": true,
                "right_click_passthrough": false
            }
        ]
    }))
    .unwrap()
}
