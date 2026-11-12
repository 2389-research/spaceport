// ABOUTME: Run native chat-start presentation checks in a private process sandbox.
// ABOUTME: Keep real window rendering and focus checks separate from agent startup.
use super::*;

#[test]
#[ignore = "requires active native desktop; chat-start presentation, no daemon"]
fn native_chat_start_errors() {
    let (status, log) =
        run_fixture(&Fixture::ChatStart).expect("native chat-start fixture timed out");
    assert_gui_success(status, &log);
    assert!(log.contains("CHAT START native PASS:"), "{log}");
}
