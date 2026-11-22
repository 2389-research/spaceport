//! The platform-delivery collisions the default table answers: a backend that
//! reports Shift as the symbol it types, Windows' AltGr arriving as Ctrl-Alt,
//! and the keys the programs inside a pane already claim.

use super::*;

/// A backend reports Shift with a digit as the symbol the digit types, so the
/// physical Ctrl-Shift-1 arrives as Ctrl-`!` and never as `ctrl-shift-1`.
#[test]
fn a_shifted_digit_arrives_as_its_symbol_so_tab_numbers_bind_both_spellings() {
    let symbols = ["!", "@", "#", "$", "%", "^", "&", "*", "("];
    for (index, symbol) in symbols.into_iter().enumerate() {
        let number = index as u8 + 1;
        let command = Command::TabNumber(number);
        let digit = format!("ctrl-shift-{number}");
        let alias = format!("ctrl-{symbol}");
        assert_eq!(linux::defaults(command), [digit.as_str(), alias.as_str()]);
        assert_eq!(windows::defaults(command), [digit.as_str(), alias.as_str()]);
        assert_eq!(macos::defaults(command), [format!("cmd-{number}")]);
    }
}

/// The same delivery for punctuation: Ctrl-Shift-`[` arrives as Ctrl-`{`.
#[test]
fn a_shifted_bracket_arrives_as_its_brace_so_each_row_names_both_spellings() {
    for (command, shifted, symbol) in [
        (Command::Back, "ctrl-shift-[", "ctrl-{"),
        (Command::Forward, "ctrl-shift-]", "ctrl-}"),
        (Command::NextPane, "ctrl-alt-shift-]", "ctrl-alt-}"),
        (Command::PreviousPane, "ctrl-alt-shift-[", "ctrl-alt-{"),
        (Command::Settings, "ctrl-shift-,", "ctrl-<"),
        (Command::Keybinds, "ctrl-shift-/", "ctrl-?"),
        (Command::SplitEditor, "ctrl-shift-\\", "ctrl-|"),
        (Command::CycleWindows, "ctrl-shift-`", "ctrl-~"),
    ] {
        let linux = linux::defaults(command);
        assert!(linux.contains(&shifted), "{command:?}: {linux:?}");
        assert!(linux.contains(&symbol), "{command:?}: {linux:?}");
    }
}

/// htop sorts with F6 and mc moves with F6 and Shift-F6, so bare F6 and F11
/// stay theirs and the app keeps the F6 name under its own Ctrl prefix.
#[test]
fn pane_programs_keep_bare_function_keys_so_the_app_prefixes_its_own() {
    assert_eq!(linux::defaults(Command::FocusNextRegion), ["ctrl-f6"]);
    assert_eq!(
        linux::defaults(Command::FocusPreviousRegion),
        ["ctrl-shift-f6"]
    );
    assert_eq!(
        linux::defaults(Command::ToggleFullScreen),
        ["ctrl-shift-f11"]
    );
    for command in [
        Command::FocusNextRegion,
        Command::FocusPreviousRegion,
        Command::ToggleFullScreen,
    ] {
        for shortcut in linux::defaults(command) {
            let key = parse(shortcut);
            assert!(key.modifiers.control, "{shortcut}");
        }
    }
}

/// A pane program pages its own output with Ctrl-PageDown and Ctrl-PageUp.
#[test]
fn pane_programs_keep_ctrl_pagedown_so_tab_cycling_takes_the_app_shift() {
    assert_eq!(linux::defaults(Command::NextTab), ["ctrl-shift-pagedown"]);
    assert_eq!(linux::defaults(Command::PreviousTab), ["ctrl-shift-pageup"]);
}

/// Windows reports AltGr as Ctrl-Alt, so a default of Ctrl-Alt and a letter
/// takes the letter an AltGr layout types: AltGr-N is `ń` on Polish, AltGr-Z
/// is `ż`. No Windows default pairs the two alone.
#[test]
fn windows_altgr_types_a_letter_so_no_default_is_ctrl_alt_and_a_letter() {
    let ctrl_alt = Modifiers {
        control: true,
        alt: true,
        ..Modifiers::default()
    };
    for info in COMMANDS {
        for shortcut in windows::defaults(info.command) {
            let key = parse(shortcut);
            let letter =
                key.key.len() == 1 && key.key.bytes().all(|byte| byte.is_ascii_lowercase());
            assert!(!(key.modifiers == ctrl_alt && letter), "{shortcut}");
        }
    }
    // The four plain Ctrl-Alt letters moved off it, keeping their names.
    assert_eq!(windows::defaults(Command::NewWindow), ["ctrl-shift-n"]);
    assert_eq!(windows::defaults(Command::ToggleZenMode), ["ctrl-shift-z"]);
    assert_eq!(
        windows::defaults(Command::ToggleStatusBar),
        ["ctrl-alt-shift-b"]
    );
    // Ctrl-Alt-Shift-W is Close Tab, and Windows closes a window with Alt+F4.
    assert!(windows::defaults(Command::CloseWindow).is_empty());
    // Linux reports AltGr as a third level, so both keep the shorter chord.
    assert_eq!(linux::defaults(Command::ToggleStatusBar), ["ctrl-alt-b"]);
    assert_eq!(linux::defaults(Command::CloseWindow), ["ctrl-alt-w"]);
}

/// Ctrl-Alt-Shift with an arrow, and Ctrl-Alt-Shift-R, may be a GNOME key.
/// No layout or desktop was measured, so these wait for a real keyboard.
#[test]
fn ctrl_alt_shift_arrows_and_r_wait_for_a_real_keyboard() {
    assert_eq!(linux::defaults(Command::FocusLeft), ["ctrl-alt-shift-left"]);
    assert_eq!(linux::defaults(Command::FocusDown), ["ctrl-alt-shift-down"]);
    assert_eq!(
        linux::defaults(Command::ResumePalette),
        ["ctrl-alt-shift-r"]
    );
}
