// ABOUTME: Checks the native profile counter through real headless layer renders.
// ABOUTME: Distinguishes render-call observations from physical display frame timing.
use crate::sidebar::layout_tests::{fixture_window, run_frames};
use std::time::Duration;

#[path = "timing.rs"]
mod timing;

#[gpui::test]
fn profile_counter_advances_when_the_layer_renders(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let layer = cx.update(|_, cx| view.read(cx).mark_layer.clone());
    let before = cx.update(|_, cx| layer.read(cx).profile_render_count());
    run_frames(cx, 4, Duration::from_millis(150));
    let after = cx.update(|_, cx| layer.read(cx).profile_render_count());
    assert!(
        after >= before + 4,
        "render counter did not advance: {before} -> {after}"
    );
}
