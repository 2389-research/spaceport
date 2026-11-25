// ABOUTME: Checks platform shortcut routing and preserves terminal and system keys.
// ABOUTME: Verifies actual catalog bindings without assuming equal platform counts.
use super::*;

#[test]
fn function_keys_can_be_configured_without_taking_ordinary_typing() {
    let keymap = with(&[("toggle_full_screen", one("f11"))]).unwrap();
    assert!(keymap.triggers(Command::ToggleFullScreen, &keystroke("f11"), false));
    for key in ["f0", "f25", "escape", "space", "shift-t"] {
        assert!(matches!(
            with(&[("new_tab", one(key))]),
            Err(Error::KeystrokeWithoutModifier { .. })
        ));
    }
}

#[cfg(target_os = "linux")]
#[test]
fn linux_shortcuts_preserve_terminal_keys_and_shifted_actions() {
    let keymap = Keymap::default();
    for (command, key) in [
        (Command::Tab, "ctrl-shift-t"),
        (Command::NewWindow, "ctrl-shift-n"),
        (Command::CloseWindow, "ctrl-alt-w"),
        (Command::Workspace, "ctrl-alt-shift-n"),
        (Command::Palette, "ctrl-shift-p"),
        (Command::ResumePalette, "ctrl-alt-shift-r"),
        (Command::SplitRight, "ctrl-shift-d"),
        (Command::SplitDown, "ctrl-alt-shift-d"),
        (Command::CopyMode, "ctrl-alt-shift-c"),
        (Command::IncreaseFontSize, "ctrl-shift-="),
        (Command::DecreaseFontSize, "ctrl-shift--"),
        (Command::ResetFontSize, "ctrl-shift-0"),
        (Command::ToggleFullScreen, "ctrl-shift-f11"),
    ] {
        assert_eq!(keymap.primary(command), key);
    }
    // Linux's backend drops Shift for punctuation while keeping its symbol.
    for (command, key) in [
        (Command::IncreaseFontSize, "ctrl-+"),
        (Command::DecreaseFontSize, "ctrl-_"),
        (Command::ResetFontSize, "ctrl-)"),
    ] {
        assert!(keymap.triggers(command, &keystroke(key), false), "{key}");
    }
    for key in [
        "ctrl-c",
        "ctrl-d",
        "ctrl-z",
        "ctrl-v",
        "ctrl-shift-c",
        "ctrl-shift-v",
        "ctrl-shift-u", // GNOME/IBus Unicode input must keep this chord.
        "cmd-t",
    ] {
        assert!(
            COMMANDS
                .iter()
                .all(|info| !keymap.triggers(info.command, &keystroke(key), false)),
            "{key}"
        );
    }
    assert!(keymap.triggers(Command::Keybinds, &keystroke("ctrl-shift-/->?"), false));
    let configured = with(&[("new_tab", one("cmd-t"))]).unwrap();
    assert_eq!(configured.primary(Command::Tab), "cmd-t");
    assert!(!configured.triggers(Command::Tab, &keystroke("ctrl-shift-t"), false));
}

#[cfg(windows)]
#[test]
fn windows_shortcuts_preserve_terminal_and_system_keys() {
    let keymap = Keymap::default();
    for (command, key) in [
        (Command::Tab, "ctrl-shift-t"),
        (Command::NewWindow, "ctrl-shift-n"),
        (Command::Palette, "ctrl-shift-p"),
        (Command::SplitRight, "ctrl-shift-d"),
        (Command::CopyMode, "ctrl-alt-shift-c"),
        (Command::ToggleStatusBar, "ctrl-alt-shift-b"),
        (Command::ResetFontSize, "ctrl-0"),
        (Command::Quit, "ctrl-shift-q"),
        (Command::ToggleFullScreen, "ctrl-shift-f11"),
    ] {
        assert_eq!(keymap.primary(command), key);
    }
    for key in [
        "ctrl-c",
        "ctrl-d",
        "ctrl-z",
        "ctrl-v",
        "ctrl-shift-c",
        "ctrl-shift-v",
        "ctrl-shift-0", // Windows can take this for input-language switching.
        "alt-f4",
        "cmd-d",
        "cmd-q",
    ] {
        assert!(
            COMMANDS
                .iter()
                .all(|info| !keymap.triggers(info.command, &keystroke(key), false)),
            "{key}"
        );
    }
    let configured = with(&[("new_tab", one("cmd-t"))]).unwrap();
    assert_eq!(configured.primary(Command::Tab), "cmd-t");
    assert!(!configured.triggers(Command::Tab, &keystroke("ctrl-shift-t"), false));
}
