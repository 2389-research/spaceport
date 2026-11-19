// ABOUTME: Avoids unmapped shifted symbols and GNOME desktop collisions in Linux defaults.
// ABOUTME: Retains the shared Ctrl profile elsewhere and a direct palette-resume shortcut.
use crate::Command;

pub(super) fn defaults(command: Command) -> &'static [&'static str] {
    match command {
        Command::ResumePalette => &["ctrl-shift-r"],
        // GNOME reserves Ctrl-Alt-Shift-arrows for moving windows between workspaces.
        // Herdr's existing prefix then h/j/k/l routes still focus panes.
        Command::FocusLeft | Command::FocusRight | Command::FocusUp | Command::FocusDown => &[],
        // Linux drops Shift from symbols without exposing their unshifted layout key.
        // Prefix routes, the palette, and explicit configuration remain available.
        Command::NextPane
        | Command::PreviousPane
        | Command::Back
        | Command::Forward
        | Command::TabNumber(_)
        | Command::Settings
        | Command::Keybinds
        | Command::SplitEditor
        | Command::CycleWindows => &[],
        _ => crate::catalog::linux::defaults(command),
    }
}
