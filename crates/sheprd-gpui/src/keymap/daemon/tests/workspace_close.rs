// ABOUTME: Preserves Herdr TUI's confirmed workspace-close prefix chord.
// ABOUTME: Allows private shortcut configuration to replace or disable that action.
use super::*;

#[test]
fn workspace_close_uses_the_herdr_chord_and_honors_explicit_overrides() {
    assert_eq!(
        bound(&DaemonKeys::default(), Command::CloseWorkspace),
        [prefixed("shift-d")]
    );
    let configured = keys("[keys]\nclose_workspace = 'prefix+shift+q'");
    assert_eq!(
        bound(&configured, Command::CloseWorkspace),
        [prefixed("shift-q")]
    );
    let disabled = keys("[keys]\nclose_workspace = ''");
    assert!(bound(&disabled, Command::CloseWorkspace).is_empty());
}
