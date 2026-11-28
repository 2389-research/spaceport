#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use spaceport_client::protocol::ClientShellSnapshot;

// keep-sorted start
mod follow;
mod host;
mod order;
mod wake;
// keep-sorted end

/// The protocol's own endpoint snapshot, for the tests that move its focus or
/// its tabs.
fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}
