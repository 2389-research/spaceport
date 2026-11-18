use crate::Command;

/// Windows reserves the Win key for the shell, and Windows Terminal leaves
/// plain Ctrl to terminal programs with the same Ctrl-Shift conventions as
/// Linux. Start from the shared Ctrl catalog and override Windows differences.
/// Linux's unmapped-symbol fallback does not apply to Windows' platform mapper.
pub(super) fn defaults(command: Command) -> &'static [&'static str] {
    match command {
        // Linux's extra symbol aliases match its backend's Shift removal.
        Command::IncreaseFontSize => &["ctrl-shift-=", "ctrl-shift-+"],
        Command::DecreaseFontSize => &["ctrl-shift--"],
        // Windows can claim Ctrl-Shift-0 as an input-language hotkey, so the
        // keystroke never arrives; Windows Terminal and browsers use Ctrl-0.
        Command::ResetFontSize => &["ctrl-0"],
        _ => crate::catalog::linux::defaults(command),
    }
}
