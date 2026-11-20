// ABOUTME: Avoids GNOME desktop collisions in Linux defaults.
// ABOUTME: Retains the shared Ctrl profile elsewhere and a direct palette-resume shortcut.
use crate::Command;

pub(super) fn defaults(command: Command) -> &'static [&'static str] {
    match command {
        Command::ResumePalette => &["ctrl-shift-r"],
        // GNOME reserves Ctrl-Alt-Shift-arrows for moving windows between workspaces.
        // Herdr's existing prefix then h/j/k/l routes still focus panes.
        Command::FocusLeft | Command::FocusRight | Command::FocusUp | Command::FocusDown => &[],
        _ => crate::catalog::linux::defaults(command),
    }
}
