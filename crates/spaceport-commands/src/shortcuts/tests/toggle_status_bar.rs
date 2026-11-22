use super::super::{linux, macos, windows};
use crate::Command;

#[test]
fn toggle_status_bar_has_a_default_shortcut_on_every_platform() {
    assert_eq!(macos::defaults(Command::ToggleStatusBar), ["cmd-alt-b"]);
    assert_eq!(linux::defaults(Command::ToggleStatusBar), ["ctrl-alt-b"]);
    // Windows reports AltGr as Ctrl-Alt, so Ctrl-Alt-B would type the `b` an
    // AltGr layout gives it; AltGr-Shift-B types nothing, and the second tier
    // keeps the sidebar's key.
    assert_eq!(
        windows::defaults(Command::ToggleStatusBar),
        ["ctrl-alt-shift-b"]
    );
}
