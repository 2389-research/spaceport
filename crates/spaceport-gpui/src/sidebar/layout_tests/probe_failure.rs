//! The probe-failure fixture still hands the native run a failure it can report.
use super::*;

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

/// `native_sidebar_probe_failure_exits_without_aborting` holds a run to this
/// log: a `native paint probe "sidebar-child"` whose `child label width: actual`
/// misses its `expected` column, no panic, and no `SIDEBAR native PASS:`. The
/// fixture asks for that failure through [`ExpectDefaultChildColumn`], so this
/// proves both halves headlessly: a narrow sidebar paints a child row the probe
/// accepts, and the same frame fails it once the fixture asks.
#[gpui::test]
fn the_probe_failure_fixture_still_fails_a_child_row_column(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(size(px(800.), px(600.)));
    view.update(cx, |view, cx| {
        view.sidebar_width = Some(180.);
        cx.notify();
    });
    let frame_at_180 = |cx: &mut gpui::VisualTestContext| {
        cx.update(|_, cx| {
            cx.set_global(PaintedProbes::default());
            cx.set_global(VerifyChildGeometry(true));
        });
        draw(cx);
        cx.update(|_, cx| cx.default_global::<PaintedProbes>().check())
    };

    // The narrow sidebar is painted correctly, so the fixture needs its switch
    // to produce the failure its run reports.
    if let Err(error) = frame_at_180(cx) {
        panic!("a 180px sidebar painted a child row the probe rejected: {error:#}");
    }
    cx.update(|_, cx| cx.set_global(ExpectDefaultChildColumn(true)));
    let Err(error) = frame_at_180(cx) else {
        panic!("the injected run painted a child row the probe accepted");
    };
    let message = format!("{error:#}");
    for expected in [
        "native paint probe \"sidebar-child\"",
        "child label width: actual",
        "expected",
    ] {
        assert!(
            message.contains(expected),
            "the injected failure does not say {expected:?}: {message}"
        );
    }
}
