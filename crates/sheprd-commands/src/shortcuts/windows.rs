use crate::Command;

/// Windows reserves the Win key for the shell, and Windows Terminal leaves
/// plain Ctrl to terminal programs with the same Ctrl-Shift conventions as
/// Linux. Start from the shared Ctrl catalog and override Windows differences.
/// Linux's GNOME exclusions do not apply to Windows.
/// Character-producing AltGr events retain the app's character-input guard.
pub(super) fn defaults(command: Command) -> &'static [&'static str] {
    match command {
        // Linux's extra symbol aliases match its backend's Shift removal.
        Command::IncreaseFontSize => &["ctrl-shift-=", "ctrl-shift-+"],
        Command::DecreaseFontSize => &["ctrl-shift--"],
        // Windows can claim Ctrl-Shift-0 as an input-language hotkey, so the
        // keystroke never arrives; Windows Terminal and browsers use Ctrl-0.
        Command::ResetFontSize => &["ctrl-0"],
        Command::ToggleStatusBar => &["ctrl-alt-shift-b"],
        // Windows closes a window with Alt+F4; Ctrl-Alt-Shift-W closes a tab.
        Command::CloseWindow => &[],
        _ => crate::catalog::linux::defaults(command),
    }
}
