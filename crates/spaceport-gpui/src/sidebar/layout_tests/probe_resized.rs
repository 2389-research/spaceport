//! A frame painted while the sidebar is resized is verified at its own width.
use super::*;

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

/// The width the frame painted the child row's label at, if the whole row was
/// inside the list's mask.
fn child_width(cx: &mut gpui::VisualTestContext) -> Option<Pixels> {
    cx.update(|_, cx| {
        cx.default_global::<PaintedProbes>()
            .0
            .get("sidebar-child")
            .map(|probe| probe.bounds.size.width)
    })
}

/// Paints one frame of a sidebar `width` wide in a `window` this size and
/// returns the child row's painted label, holding that frame to its own column.
fn painted_child(
    cx: &mut gpui::TestAppContext,
    window: gpui::Size<Pixels>,
    width: Option<f32>,
) -> Pixels {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|_, cx| {
        cx.set_global(PaintedProbes::default());
        cx.set_global(VerifyChildGeometry(true));
    });
    cx.simulate_resize(window);
    view.update(cx, |view, cx| {
        view.sidebar_width = width;
        cx.notify();
    });
    // A notify can draw before this frame: only this frame's probes count, and
    // the whole probe global starts over, first error included.
    cx.update(|_, cx| cx.set_global(PaintedProbes::default()));
    draw(cx);
    let Some(painted) = child_width(cx) else {
        panic!("the frame in a {window:?} window at {width:?} did not paint the child row");
    };
    if let Err(error) = cx.update(|_, cx| cx.default_global::<PaintedProbes>().check()) {
        panic!("child row in a {window:?} window at {width:?}: {error:#}");
    }
    painted
}

/// The child row's column comes from the width the frame measured, so a sidebar
/// resized by its edge, or clamped part-way through a window resize, is held to
/// what it painted. Measuring every frame against the default width left those
/// frames unverified, and the native fixture skipped them rather than fail.
#[gpui::test]
fn a_resized_sidebar_verifies_its_child_row_at_its_own_width(cx: &mut gpui::TestAppContext) {
    let default = painted_child(cx, size(px(800.), px(600.)), None);
    assert_eq!(default, px(163.), "the default-width child column");
    // The edge dragged wider: the same row takes the wider column.
    let wider = painted_child(cx, size(px(800.), px(600.)), Some(320.));
    assert_eq!(wider, default + px(88.), "the 320px child column");
    // A window too narrow for the default width clamps the sidebar, which is the
    // frame a resize paints before it settles.
    assert!(painted_child(cx, size(px(420.), px(600.)), None) < default);
}
