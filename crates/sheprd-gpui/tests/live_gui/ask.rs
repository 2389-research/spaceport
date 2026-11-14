// ABOUTME: Verifies the real native Ask Spaceport view in an isolated app process.
// ABOUTME: Keeps presentation and focus checks separate from provider qualification.
use super::*;

#[test]
#[ignore = "requires native rendering; isolated Ask presentation, no daemon or provider"]
fn native_ask_spaceport() {
    let (status, log) = run_fixture(&Fixture::Ask).expect("native Ask fixture timed out");
    assert_gui_success(status, &log);
    assert!(log.contains("ASK native PASS:"), "{log}");
}
