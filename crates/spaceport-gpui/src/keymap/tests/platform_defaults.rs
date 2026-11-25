// ABOUTME: Checks the app's reserved keys against the catalog defaults.
// ABOUTME: Every default is its own keystroke and none takes an Edit or system key.
use super::*;
use crate::controls::{COMMANDS, CTRL_SHORTCUTS};

#[test]
fn platform_defaults_are_unique_and_leave_edit_keys_available() {
    let keymap = with(&[]).unwrap();
    let mut seen = HashMap::new();
    for info in COMMANDS {
        // Catalog tests cover intentional platform differences, including unbound keys.
        for shortcut in info.default_shortcuts() {
            let key = keystroke(shortcut);
            assert!(
                seen.insert(identity(&key), info.name).is_none(),
                "{shortcut}"
            );
            assert!(keymap.triggers(info.command, &key, false), "{shortcut}");
            assert!(
                crate::actions::reserved_keystrokes()
                    .all(|edit| !typed_matches(&key, &keystroke(edit))),
                "{shortcut}"
            );
            if CTRL_SHORTCUTS {
                assert!(!key.modifiers.platform, "{shortcut}");
            } else {
                assert!(key.modifiers.platform, "{shortcut}");
            }
        }
    }
}
