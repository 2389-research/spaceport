// ABOUTME: Checks that device setup preserves the manual target without discovery.
// ABOUTME: Uses an explicitly enabled Core window in the private fixture.
use crate::sidebar::layout_tests::fixture_core_window;
use gpui::{VisualTestContext, px, size};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

#[gpui::test]
fn add_device_keeps_manual_target_and_has_no_discovery(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.simulate_resize(size(px(800.), px(700.)));
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_add_device(window, cx)));
    draw(cx);
    assert!(cx.debug_bounds("device-setup-dialog").is_some());
    assert!(cx.debug_bounds("device-discovery").is_none());

    cx.update(|window, cx| {
        let fields = view
            .read(cx)
            .menu
            .device_setup
            .as_ref()
            .unwrap()
            .fields
            .clone();
        assert!(fields[0].read(cx).focus.is_focused(window));
        fields[0].update(cx, |field, cx| {
            field.set_text_selected("me@lab.example", cx)
        });
        fields[1].update(cx, |field, cx| field.set_text_selected("Lab", cx));
    });
    draw(cx);
    cx.update(|_, cx| {
        let setup = view.read(cx).menu.device_setup.as_ref().unwrap();
        assert_eq!(setup.fields[0].read(cx).text(), "me@lab.example");
        assert_eq!(setup.fields[1].read(cx).text(), "Lab");
    });
    assert!(cx.debug_bounds("device-discovery").is_none());
}
