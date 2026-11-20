// ABOUTME: Keeps scrolled device-theme choices visible after active owners drop.
// ABOUTME: Checks focus restoration without retaining the host's native input.
use super::*;
use crate::menu::devices::device_theme_departure_fixture;
use gpui::{ScrollDelta, ScrollEvent, TestAppContext};

#[gpui::test]
fn device_theme_rows_leave_after_the_host_and_input_drop(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(device_theme_departure_fixture);
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let row = cx
        .debug_bounds("device-theme-row-5")
        .expect("built-in choice");
    cx.simulate_event(ScrollEvent {
        position: row.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let scrolled = cx
        .debug_bounds("device-theme-row-5")
        .expect("scrolled choice");
    assert!(
        scrolled.top() < row.top(),
        "the live choices actually scroll"
    );
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            assert!(view.menu.host.is_none());
            assert!(view.focus.is_focused(window));
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view
                .menu
                .appear
                .leaving
                .as_mut()
                .expect("device theme leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        })
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let leaving = cx
        .debug_bounds("device-theme-row-5")
        .expect("departing choice");
    assert!((leaving.top() - scrolled.top()).abs() < px(1.));
}
