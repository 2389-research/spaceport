// ABOUTME: Provides a header fixture and checks sidebar toggle behavior.
// ABOUTME: Shares the fixture with geometry and profile-removal tests.
#![allow(clippy::unwrap_used)]
use crate::menu::Page;
use gpui::{Context, Modifiers, TestAppContext, Window, px, size};

mod without_profile;

/// The fixture window with Herdr's tab bar at the bottom, so it keeps the
/// full-width header these tests measure.
pub(crate) fn header_window(
    window: &mut Window,
    cx: &mut Context<crate::SheprdWindow>,
) -> crate::SheprdWindow {
    let mut view = crate::sidebar::layout_tests::fixture_window(window, cx);
    view.settings.shared = Some(
        crate::herdr_settings::Settings::parse_text("[ui]\ntab_bar_position = 'bottom'").unwrap(),
    );
    view
}

#[gpui::test]
fn sidebar_button_collapses_and_reopens_without_moving(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(header_window);
    for width in [1200., 360.] {
        cx.simulate_resize(size(px(width), px(600.)));
        for visible in [true, false] {
            cx.update(|window, cx| {
                window.refresh();
                window.draw(cx).clear(cx);
            });
            assert_eq!(view.read_with(cx, |view, _| view.sidebar_visible), visible);
            assert_eq!(cx.debug_bounds("sidebar").is_some(), visible);
            let button = cx.debug_bounds("toggle-sidebar").unwrap();
            assert_eq!(button.origin.x, px(super::LEADING));
            assert_eq!(button.size, size(px(28.), px(28.)));
            cx.simulate_click(button.center(), Modifiers::default());
        }
        assert!(view.read_with(cx, |view, _| view.sidebar_visible));
    }
}
