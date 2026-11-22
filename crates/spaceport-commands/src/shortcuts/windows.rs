use crate::Command;

/// Windows reserves the Win key for the shell, and Windows Terminal leaves
/// plain Ctrl to terminal programs with the same Ctrl-Shift conventions as
/// Linux. Start from the Linux profile and override here only where Windows
/// itself differs, so a Linux change carries over unless Windows opts out.
///
/// One such difference is AltGr: Windows reports it as Ctrl-Alt, while Linux
/// reports it as a third level and keeps Ctrl-Alt for the app. A default bound
/// to Ctrl-Alt and a letter alone therefore takes the letter an AltGr layout
/// types, as `ń` on Polish, so the Windows profile turns those into its second
/// tier with Shift, where an AltGr layout types nothing.
pub(super) fn defaults(command: Command) -> &'static [&'static str] {
    match command {
        // Linux's extra symbol aliases match its backend's Shift removal.
        Command::IncreaseFontSize => &["ctrl-shift-=", "ctrl-shift-+"],
        Command::DecreaseFontSize => &["ctrl-shift--"],
        // Windows can claim Ctrl-Shift-0 as an input-language hotkey, so the
        // keystroke never arrives; Windows Terminal and browsers use Ctrl-0.
        Command::ResetFontSize => &["ctrl-0"],
        // Ctrl-Alt-B is AltGr-B, which types a letter on an AltGr layout. The
        // second tier keeps the key: AltGr-Shift-B types nothing.
        Command::ToggleStatusBar => &["ctrl-alt-shift-b"],
        // Ctrl-Alt-W is AltGr-W, which types `?` on a Brazilian ABNT2 layout,
        // and Ctrl-Alt-Shift-W is Close Tab here. Windows closes a window with
        // Alt+F4 itself, so this one keeps no chord.
        Command::CloseWindow => &[],
        _ => super::linux::defaults(command),
    }
}
