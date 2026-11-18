// ABOUTME: Verifies retained Herdr prefix routes for commands without Linux symbol defaults.
// ABOUTME: Exercises the real parsed view keymap rather than assuming catalog availability.
use super::*;

#[test]
fn prefix_routes_keep_panes_tabs_settings_and_help_available() {
    let keymap = crate::keymap::Keymap::default();
    assert!(keymap.is_prefix(&parsed("ctrl-b")));
    for (key, command) in [
        ("tab", Command::NextPane),
        ("shift-tab", Command::PreviousPane),
        ("s", Command::Settings),
        ("?", Command::Keybinds),
    ] {
        assert_eq!(keymap.chord(&parsed(key)), Some(command), "{key}");
    }
    for number in 1..=9 {
        assert_eq!(
            keymap.chord(&parsed(&number.to_string())),
            Some(Command::TabNumber(number))
        );
    }
}
