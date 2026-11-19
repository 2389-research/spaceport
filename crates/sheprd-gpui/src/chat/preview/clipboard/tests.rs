// ABOUTME: Checks standalone fixture wiring through the actual multiline keyboard owner.
// ABOUTME: Keeps headless routing evidence separate from native clipboard acquisition.
#![allow(clippy::unwrap_used)]
use super::*;
use gpui::{ClipboardItem, KeyDownEvent, KeyUpEvent};

#[gpui::test]
fn standalone_fixture_pastes_through_its_field_without_chat_capture(cx: &mut gpui::TestAppContext) {
    crate::actions::bind_keys(&mut cx.app.borrow_mut());
    let (view, cx) = cx.add_window_view(|_, cx| Field::new(cx));
    let input = view.read_with(cx, |view, _| view.input.clone());
    cx.update(|window, cx| {
        let focus = input.read(cx).focus.clone();
        focus.focus(window, cx);
        window.draw(cx).clear(cx);
        cx.write_to_clipboard(ClipboardItem::new_string("Authored\r\né界🦀".into()));
    });
    let key = if cfg!(target_os = "macos") {
        "cmd-v"
    } else {
        "ctrl-v"
    };
    let keystroke = crate::keymap::parse_keystroke(key).unwrap();
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
    cx.run_until_parked();
    input.read_with(cx, |input, _| {
        assert_eq!(input.text(), "Authored\né界🦀");
        assert!(!input.is_pasting());
    });
}
