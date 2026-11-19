use super::super::{linux, macos, windows};
use crate::Command;

#[test]
fn toggle_status_bar_has_a_default_shortcut_on_every_platform() {
    assert_eq!(macos::defaults(Command::ToggleStatusBar), ["cmd-alt-b"]);
    assert_eq!(linux::defaults(Command::ToggleStatusBar), ["ctrl-alt-b"]);
    // Windows keeps this on its second tier; character-producing AltGr events
    // still depend on the app's character-input guard.
    assert_eq!(
        windows::defaults(Command::ToggleStatusBar),
        ["ctrl-alt-shift-b"]
    );
}
