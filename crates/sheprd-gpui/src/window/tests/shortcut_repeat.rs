// ABOUTME: Exercises shortcut repeat handling through real GPUI key dispatch.
// ABOUTME: Preserves repeated font changes and ordinary terminal key delivery.
use super::*;
#[cfg(feature = "integration-test")]
mod daemon;
mod text_input;

fn press(cx: &mut gpui::VisualTestContext, key: &str, held: bool) -> bool {
    let result = cx.update(|window, cx| {
        window.dispatch_event(
            gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                keystroke: crate::keymap::parse_keystroke(key).unwrap(),
                is_held: held,
                prefer_character_input: false,
            }),
            cx,
        )
    });
    cx.run_until_parked();
    !result.propagate
}

#[gpui::test]
fn direct_toggle_runs_once_until_a_fresh_press(cx: &mut gpui::TestAppContext) {
    cx.update(crate::actions::bind_keys);
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
    let key = view.read_with(cx, |view, _| {
        view.keymap().primary(Command::ToggleSidebar).to_owned()
    });
    press(cx, &key, false);
    assert!(!view.read_with(cx, |view, _| view.sidebar_visible));
    press(cx, &key, true);
    assert!(!view.read_with(cx, |view, _| view.sidebar_visible));
    press(cx, &key, false);
    assert!(view.read_with(cx, |view, _| view.sidebar_visible));
}

#[gpui::test]
fn holding_the_prefix_does_not_consume_its_chord(cx: &mut gpui::TestAppContext) {
    cx.update(crate::actions::bind_keys);
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
    press(cx, "ctrl-b", false);
    assert!(view.read_with(cx, |view, _| view.prefix_armed));
    press(cx, "ctrl-b", true);
    assert!(view.read_with(cx, |view, _| view.prefix_armed));
    press(cx, "b", false);
    assert!(!view.read_with(cx, |view, _| view.sidebar_visible));
    assert!(press(cx, "b", true));
    assert!(!view.read_with(cx, |view, _| view.sidebar_visible));
}

#[gpui::test]
fn font_shortcuts_keep_repeating(cx: &mut gpui::TestAppContext) {
    cx.update(crate::actions::bind_keys);
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
    let (key, before) = view.read_with(cx, |view, _| {
        (
            view.keymap().primary(Command::IncreaseFontSize).to_owned(),
            view.config.terminal.size,
        )
    });
    press(cx, &key, false);
    let first = view.read_with(cx, |view, _| view.config.terminal.size);
    press(cx, &key, true);
    let repeated = view.read_with(cx, |view, _| view.config.terminal.size);
    assert!(first > before && repeated > first);
}

#[gpui::test]
fn holding_new_window_opens_one_window_until_a_fresh_press(cx: &mut gpui::TestAppContext) {
    cx.update(crate::actions::bind_keys);
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
    let key = view.read_with(cx, |view, _| {
        view.keymap().primary(Command::NewWindow).to_owned()
    });
    let before = cx.update(|_, cx| main_windows(cx).len());
    press(cx, &key, false);
    assert_eq!(cx.update(|_, cx| main_windows(cx).len()), before + 1);
    press(cx, &key, true);
    assert_eq!(cx.update(|_, cx| main_windows(cx).len()), before + 1);
    press(cx, &key, false);
    assert_eq!(cx.update(|_, cx| main_windows(cx).len()), before + 2);
}

#[gpui::test]
fn menu_actions_still_run_without_a_keyboard_event(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
        window.dispatch_action(
            Box::new(crate::actions::RunCommand {
                command: Command::ToggleSidebar,
            }),
            cx,
        );
    });
    cx.run_until_parked();
    assert!(!view.read_with(cx, |view, _| view.sidebar_visible));
}

#[gpui::test]
fn releasing_a_consumed_chord_returns_its_key_to_menu_text_input(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        window.focus(&view.read(cx).focus.clone(), cx);
        window.draw(cx).clear(cx);
    });
    press(cx, "ctrl-b", false);
    assert!(press(cx, "b", false));
    view.update_in(cx, |view, window, cx| view.open_keybinds(window, cx));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(press(cx, "b", true));
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.menu.keybinds_search.as_ref().unwrap().read(cx).text(),
            ""
        )
    });
    cx.update(|window, cx| {
        window.dispatch_event(
            gpui::PlatformInput::KeyUp(gpui::KeyUpEvent {
                keystroke: crate::keymap::parse_keystroke("b").unwrap(),
            }),
            cx,
        );
    });
    cx.update(|window, cx| {
        window.dispatch_keystroke(crate::keymap::parse_keystroke("b").unwrap(), cx);
    });
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.menu.keybinds_search.as_ref().unwrap().read(cx).text(),
            "b"
        )
    });
}
