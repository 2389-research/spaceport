#![allow(clippy::unwrap_used)]
//! The ranking, the status filter's heading order, the fences a captured
//! destination answers to, and the scoring a row is found by.

use spaceport_client::protocol::ClientShellSnapshot;

mod agent_filter;
mod rank;
mod search;
mod target;

/// The endpoint snapshot the fence tests work from: one workspace, one tab and
/// its panes, and a daemon process the commands run against.
pub(crate) fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}
