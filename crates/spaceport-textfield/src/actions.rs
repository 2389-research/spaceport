//! The edit actions a text field registers, and the keystroke an Edit menu
//! item stands for.

use gpui::{KeyDownEvent, Keystroke, Modifiers, actions};
use spaceport_commands::CTRL_SHORTCUTS;

// Edit menu items. Each element registers only the ones it can perform, so
// GPUI disables the rest while that element has focus.
actions!(edit, [Cut, Copy, Paste, SelectAll]);

/// The keystroke an Edit menu item stands for. A menu click replays it
/// through the focused element's own key handler, so choosing the item and
/// pressing its shortcut cannot behave differently.
pub fn edit_key(key: &str) -> KeyDownEvent {
    KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers {
                platform: !CTRL_SHORTCUTS,
                control: CTRL_SHORTCUTS,
                shift: CTRL_SHORTCUTS,
                ..Modifiers::default()
            },
            key: key.into(),
            key_char: None,
        },
        is_held: false,
        prefer_character_input: false,
    }
}
