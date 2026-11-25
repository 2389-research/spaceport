//! Shift-Enter reaches a pane as Alt-Enter, so a program at Herdr's legacy
//! keyboard level reads a newline chord instead of a bare CR (#504).

use super::*;
use gpui::KeyDownEvent;
use spaceport_client::protocol::{ClientKeyCode, ClientPaneInputEvent};

#[test]
fn shift_enter_sends_alt_enter_on_every_platform() {
    let keymap = Keymap::default();
    let Some(target) = sent(&keymap, "shift-enter") else {
        panic!("shift-enter has no default");
    };
    assert_eq!(target, &key("alt-enter"));
    let typed = KeyDownEvent {
        keystroke: keystroke("shift-enter"),
        is_held: false,
        prefer_character_input: false,
    };
    let Some(ClientPaneInputEvent::Key {
        code, modifiers, ..
    }) = crate::terminal::pane_key_input(&typed, target)
    else {
        panic!("no key sent");
    };
    // Alt alone: Shift would let the daemon collapse it to a bare CR again.
    assert_eq!((code, modifiers), (ClientKeyCode::Enter, 4));
    // Plain Enter and other modified Enters are untouched.
    for other in ["enter", "ctrl-enter", "alt-enter", "ctrl-shift-enter"] {
        assert_eq!(sent(&keymap, other), None, "{other}");
    }
}
