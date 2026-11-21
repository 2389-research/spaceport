// ABOUTME: Checks the redraw budget of real Working marks with GPUI's test clock.
// ABOUTME: Separates scheduled animation renders from forced draws and native frame rates.
use crate::sidebar::layout_tests::fixture_window;
use std::time::Duration;

#[gpui::test]
fn working_marks_wait_fifty_milliseconds_between_timer_redraws(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.run_until_parked();
    let layer = cx.update(|window, cx| {
        assert!(
            !cx.reduce_motion(),
            "the timer budget requires ordinary motion"
        );
        let Some(marks) = crate::mark_layer::Marks::of(window, cx) else {
            panic!("the fixture window must own a mark layer");
        };
        assert!(
            marks.len() > 0,
            "the fixture must reserve real Working marks"
        );
        view.read(cx).mark_layer.clone()
    });
    let before = cx.update(|_, cx| layer.read(cx).profile_render_count());
    assert!(
        before > 0,
        "the initial layer must paint before timing begins"
    );

    cx.executor().advance_clock(Duration::from_millis(40));
    cx.run_until_parked();
    let before_deadline = cx.update(|_, cx| layer.read(cx).profile_render_count());
    assert_eq!(before_deadline, before, "Working marks redrew before 50 ms");

    cx.executor().advance_clock(Duration::from_millis(20));
    cx.run_until_parked();
    let after_deadline = cx.update(|_, cx| layer.read(cx).profile_render_count());
    assert!(
        after_deadline > before,
        "Working marks stopped scheduling redraws"
    );
}
