// ABOUTME: Exercises keyboard activation of the connection recovery cards.
// ABOUTME: Keeps focus traversal separate from terminal input and personal daemons.
use super::*;
use crate::window::chrome_focus::Region;

fn press(cx: &mut gpui::VisualTestContext, key: &str) {
    let stroke = crate::keymap::parse_keystroke(key).unwrap();
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        window.dispatch_keystroke(stroke.clone(), cx);
        window.dispatch_event(
            gpui::PlatformInput::KeyUp(gpui::KeyUpEvent { keystroke: stroke }),
            cx,
        );
    });
    cx.run_until_parked();
}

#[gpui::test]
fn missing_herdr_actions_work_with_enter_space_and_tab(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, _| {
            view.endpoints[0].never_connect();
            view.endpoints[0].lose_herdr_for_test("fixture missing");
            view.chrome_region = Some(Region::Notices);
        });
        window.draw(cx).clear(cx);
        window.blur(cx);
        window.focus_next(cx);
    });
    press(cx, "enter");
    assert_eq!(cx.opened_url().as_deref(), Some("https://herdr.dev/tui/"));
    let generation = view.read_with(cx, |view, _| view.endpoints[0].generation);
    press(cx, "tab");
    press(cx, "space");
    view.read_with(cx, |view, _| {
        assert!(view.endpoints[0].generation > generation)
    });
}

#[gpui::test]
fn reconnect_is_a_keyboard_action(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, _| {
            view.endpoints[0].never_connect();
            view.endpoints[0].lose_connection_for_test("fixture outage");
            view.chrome_region = Some(Region::Notices);
        });
        window.draw(cx).clear(cx);
        window.blur(cx);
        window.focus_next(cx);
    });
    let generation = view.read_with(cx, |view, _| view.endpoints[0].generation);
    press(cx, "enter");
    view.read_with(cx, |view, _| {
        assert!(view.endpoints[0].generation > generation)
    });
}
