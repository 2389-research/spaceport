// ABOUTME: Checks Linux's layout-independent shortcut fallback without changing Windows defaults.
// ABOUTME: Keeps removed shifted-symbol defaults explicit and bounds the profile change.
use super::{COMMANDS, Command, windows};
use crate::catalog;

fn removed(command: Command) -> bool {
    matches!(
        command,
        Command::NextPane
            | Command::PreviousPane
            | Command::Back
            | Command::Forward
            | Command::TabNumber(_)
            | Command::Settings
            | Command::Keybinds
            | Command::SplitEditor
            | Command::CycleWindows
    )
}

#[test]
fn linux_uses_only_the_listed_platform_fallbacks() {
    for info in COMMANDS {
        let expected = if info.command == Command::ResumePalette {
            &["ctrl-shift-r"][..]
        } else if removed(info.command) || super::gnome::catalog_uses_gnome_chord(info.command) {
            &[][..]
        } else {
            catalog::linux::defaults(info.command)
        };
        assert_eq!(
            super::linux::defaults(info.command),
            expected,
            "{}",
            info.name
        );
    }
}

#[test]
fn windows_keeps_its_mapped_shifted_symbol_bindings() {
    for info in COMMANDS.iter().filter(|info| removed(info.command)) {
        assert_eq!(
            windows::defaults(info.command),
            catalog::linux::defaults(info.command),
            "{}",
            info.name
        );
    }
}
