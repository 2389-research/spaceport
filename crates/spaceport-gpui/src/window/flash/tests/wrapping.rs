// ABOUTME: Checks repair-warning layout at normal and narrow terminal widths.
// ABOUTME: Keeps long guidance within the cells while preserving short confirmation geometry.
use super::*;
use crate::{i18n::Localize, sidebar::layout_tests::full_draw};
use gpui::{px, size};

#[gpui::test]
fn saved_state_repair_guidance_wraps_inside_the_terminal(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let repair = spaceport_browser::Error::RestorationBlocked.localize();
    for width in [900., 640.] {
        cx.simulate_resize(size(px(width), px(600.)));
        cx.run_until_parked();
        view.update(cx, |view, cx| {
            view.show_flash(Flash::success("Copied"), cx);
            view.flash_motion.since = None;
        });
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let short = cx.debug_bounds("flash").expect("short flash painted");

        view.update(cx, |view, cx| {
            view.show_flash(Flash::warning(repair.clone()), cx);
        });
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let long = cx.debug_bounds("flash").expect("repair flash painted");
        let terminal = view.read_with(cx, |view, _| view.bounds);
        assert!(
            long.size.height > short.size.height,
            "repair guidance still occupies one clipped row at {width}px: {long:?}"
        );
        assert!(long.left() >= terminal.left(), "flash escaped its terminal");
        assert!(
            long.right() <= terminal.right(),
            "flash escaped its terminal"
        );
        assert!(
            long.bottom() <= terminal.bottom(),
            "flash escaped its terminal"
        );
    }
}
