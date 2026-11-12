// ABOUTME: Runs real AppKit sidebar events without requiring an unlocked desktop.
// ABOUTME: Keeps key-window qualification in the separate native_sidebar fixture.
use super::*;

#[test]
#[ignore = "requires macOS WindowServer; no unlocked desktop or daemon"]
fn native_host_routing() {
    native_fixture(Fixture::SidebarRouting);
}
