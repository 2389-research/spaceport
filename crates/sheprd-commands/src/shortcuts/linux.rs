// ABOUTME: Leaves layout-dependent shifted symbols unbound on Linux's unmapped keyboard backend.
// ABOUTME: Retains the shared Ctrl profile for other commands and explicit font aliases.
use crate::Command;

pub(super) fn defaults(command: Command) -> &'static [&'static str] {
    match command {
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
