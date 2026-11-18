// ABOUTME: Keeps held activation keys from toggling or selecting custom application menus.
// ABOUTME: Exercises the actual window capture and popup-owner key handlers.
use super::*;

#[gpui::test]
fn held_activation_keys_do_not_toggle_or_choose_an_application_menu(cx: &mut TestAppContext) {
    for (open, key) in [
        (false, "f10"),
        (true, "f10"),
        (true, "enter"),
        (true, "space"),
        (true, "escape"),
    ] {
        let (view, cx) = window(cx);
        draw(cx);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                if open {
                    view.open_application_menu(Some(0), window, cx);
                }
                let before = view.menu.page;
                let held = KeyDownEvent {
                    keystroke: crate::keymap::parse_keystroke(key).unwrap(),
                    is_held: true,
                    prefer_character_input: false,
                };
                if open {
                    view.application_key(&held, window, cx);
                } else {
                    view.application_menu_shortcut(&held, window, cx);
                }
                assert_eq!(view.menu.page, before, "held {key}");
            });
        });
    }
}
