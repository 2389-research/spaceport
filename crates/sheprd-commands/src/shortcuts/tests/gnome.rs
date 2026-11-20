// ABOUTME: Keeps Linux defaults clear of GNOME's workspace and recording bindings.
// ABOUTME: Preserves the shared Windows defaults for those same app commands.
use super::{COMMANDS, Command, linux, parse, windows};

const RESERVED: &[&str] = &[
    // keep-sorted start
    "ctrl-alt-shift-down",
    "ctrl-alt-shift-left",
    "ctrl-alt-shift-r",
    "ctrl-alt-shift-right",
    "ctrl-alt-shift-up",
    // keep-sorted end
];

fn reserved(shortcut: &str) -> bool {
    let key = parse(shortcut);
    RESERVED
        .iter()
        .map(|key| parse(key))
        .any(|other| key.modifiers == other.modifiers && key.key == other.key)
}

pub(super) fn catalog_uses_gnome_chord(command: Command) -> bool {
    crate::catalog::linux::defaults(command)
        .iter()
        .any(|shortcut| reserved(shortcut))
}

#[test]
fn linux_defaults_leave_gnome_workspace_and_recording_chords_unbound() {
    for info in COMMANDS {
        for shortcut in linux::defaults(info.command) {
            assert!(
                !reserved(shortcut),
                "{} uses desktop shortcut {shortcut}",
                info.name
            );
        }
    }
}

#[test]
fn windows_keeps_the_shared_defaults_for_gnome_reserved_commands() {
    for info in COMMANDS
        .iter()
        .filter(|info| catalog_uses_gnome_chord(info.command))
    {
        assert_eq!(
            windows::defaults(info.command),
            crate::catalog::linux::defaults(info.command),
            "{}",
            info.name
        );
    }
}

#[test]
fn linux_resume_keeps_a_direct_non_desktop_shortcut() {
    assert_eq!(linux::defaults(Command::ResumePalette), ["ctrl-shift-r"]);
}
