use super::*;
use crate::motion::{ENTER, LEAVE};
use gpui::{point, px};

mod kept;

#[test]
fn unit_tests_see_menus_settled() {
    let start = Instant::now();
    let mut appear = Appear::default();
    appear.observe(Some(Shown::Dialog), None, start);
    assert_eq!(appear.state.panel(start), 1.);
    assert_eq!(appear.state.backdrop(start), 1.);
    assert!(!appear.state.moving(start));
    appear.observe(None, None, start);
    assert_eq!(appear.state.backdrop(start), 0.);
}

#[test]
fn the_dimming_keeps_its_color_and_scales_its_alpha() {
    assert_eq!(backdrop(0x112233, 1.), rgba(0x112233b0));
    assert_eq!(backdrop(0x112233, 0.), rgba(0x11223300));
    assert_eq!(backdrop(0x112233, 2.), rgba(0x112233b0));
}

#[gpui::test]
fn all_devices_draws_lower_while_it_rises(cx: &mut gpui::TestAppContext) {
    use crate::sidebar::layout_tests::fixture_window;
    let (view, cx) = cx.add_window_view(fixture_window);
    let draw = |cx: &mut gpui::VisualTestContext| {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    };
    // The footer sits at the window's foot, so the menu opens above it.
    let foot = cx.update(|window, _| window.viewport_size().height) - px(40.);
    let open = |cx: &mut gpui::VisualTestContext| {
        view.update(cx, |view, _| {
            view.menu.anchor = point(px(20.), foot);
            view.menu.page = Some(Page::Devices);
        });
    };
    open(cx);
    draw(cx);
    let settled = cx.debug_bounds("menu-panel").expect("All Devices draws");
    view.update(cx, |view, _| view.menu.page = None);
    draw(cx);
    view.update(cx, |view, _| view.menu.appear.live = true);
    open(cx);
    draw(cx);
    // Held at its start, however long the frames take: a start still ahead
    // reads as not begun.
    view.update(cx, |view, _| {
        view.menu.appear.state.since = Some(motion::now() + std::time::Duration::from_secs(3600));
    });
    draw(cx);
    let rising = cx.debug_bounds("menu-panel").expect("All Devices draws");
    assert!(
        rising.origin.y > settled.origin.y + px(1.),
        "{rising:?} rises to {settled:?}"
    );
}

#[gpui::test]
fn the_devices_popover_is_drawn_as_it_goes_and_leaves_the_menu_reset(
    cx: &mut gpui::TestAppContext,
) {
    use crate::sidebar::layout_tests::fixture_window;
    let (view, cx) = cx.add_window_view(fixture_window);
    let draw = |cx: &mut gpui::VisualTestContext| {
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
    };
    view.update(cx, |view, _| {
        view.menu.appear.live = true;
        view.menu.anchor = point(px(200.), px(40.));
        view.menu.page = Some(Page::Devices);
        view.menu.error = Some("device failed".into());
    });
    draw(cx);
    assert!(cx.debug_bounds("menu-leaving-shield").is_none());
    cx.update(|window, cx| view.update(cx, |view, cx| view.dismiss_menu(window, cx)));
    view.update(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert_eq!(view.menu.error, None);
        // Held at the start of its way out, however long the frames take.
        // Closing may already have drawn a frame that started it, so the
        // start is set as well as observed.
        let later = motion::now() + std::time::Duration::from_secs(3600);
        view.menu.appear.state.observe(None, None, later, false);
        view.menu.appear.state.leaving = Some((Page::Devices, later));
    });
    draw(cx);
    // The copy going away was drawn, shielded, and the menu's own state is
    // as the reset left it once it was.
    assert!(cx.debug_bounds("menu-leaving-shield").is_some());
    view.update(cx, |view, _| {
        assert_eq!(view.menu.page, None);
        assert_eq!(view.menu.error, None);
        assert!(view.menu.appear.state.leaving(motion::now()).is_some());
    });
}

#[gpui::test]
fn the_dimming_lifts_behind_the_picker_a_dialog_returns_to(cx: &mut gpui::TestAppContext) {
    use crate::sidebar::layout_tests::fixture_window;
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        let start = Instant::now();
        view.menu.appear.live = true;
        view.menu
            .appear
            .state
            .observe(Some(Shown::Dialog), Some(Page::Devices), start, false);
        let open = start + ENTER;
        // Escape in the session editor goes back to the picker, a popover.
        view.menu.page = Some(Page::Devices);
        view.menu.appear.state.observe(
            Some(popover(point(px(10.), px(600.)))),
            Some(Page::Devices),
            open,
            false,
        );
        assert!(!view.menu_dims());
        assert_eq!(view.menu.appear.state.backdrop(open), 1.);
        assert!(view.menu.appear.state.moving(open));
        assert_eq!(view.menu.appear.state.backdrop(open + LEAVE), 0.);
        // The picker was drawn beneath the form all along, so it is uncovered
        // in place rather than rising out of the footer again.
        assert_eq!(view.menu.appear.state.panel(open), 1.);
    });
}
