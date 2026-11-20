// ABOUTME: Checks settled Core cell paints during actual Working-mark frames.
// ABOUTME: Uses emulator content and the real view while keeping native timing claims separate.
use super::selected;
use crate::{
    core_terminals::tests::recording_link, sidebar::layout_tests::run_frames,
    window::native_images::Session,
};
use gpui::TestAppContext;
use std::{sync::Arc, time::Duration};

#[gpui::test]
fn settled_core_cells_replay_while_working_marks_advance(cx: &mut TestAppContext) {
    let (fixture, cx) = selected(cx, false);
    let (link, _received) = recording_link();
    fixture.view.update(cx, |view, _| view.link = link);
    let frame = fixture
        .view
        .read_with(cx, |view, _| view.screen.frame.clone().unwrap());
    let probe = cx
        .update(|window, _| Session::start(window.window_handle().window_id()))
        .unwrap();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    run_frames(cx, 2, Duration::from_millis(150));
    assert!(
        probe.frame().unwrap().cells > 0,
        "the populated Core must paint"
    );
    assert!(
        fixture
            .view
            .read_with(cx, |view, _| view.bounds().is_some())
    );
    let layer = fixture
        .owner
        .read_with(cx, |owner, _| owner.mark_layer.clone());
    let before = layer.read_with(cx, |layer, _| layer.profile_render_count());
    probe.reset().unwrap();
    run_frames(cx, 4, Duration::from_millis(150));
    let after = layer.read_with(cx, |layer, _| layer.profile_render_count());
    assert!(
        after >= before + 4,
        "the Working marks must actually advance"
    );
    fixture.view.read_with(cx, |view, _| {
        assert!(view.screen.failure.is_none());
        assert!(Arc::ptr_eq(view.screen.frame.as_ref().unwrap(), &frame));
    });
    assert_eq!(
        probe.frame().unwrap().cells,
        0,
        "settled Core cells repainted during mark-only frames"
    );
}
