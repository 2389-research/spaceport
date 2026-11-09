#![allow(clippy::unwrap_used)]
use crate::menu::Page;
use gpui::{
    Bounds, Context, Modifiers, MouseButton, MouseDownEvent, TestAppContext, Window, point, px,
    size,
};

mod profile_shell;

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

#[gpui::test]
fn profile_icon_keeps_the_same_bounds_on_hover(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(header_window);
    for width in [1200., 360.] {
        cx.simulate_resize(size(px(width), px(400.)));
        for hovered in [false, true] {
            cx.update(|window, cx| {
                window.refresh();
                let _ = window.draw(cx);
            });
            let hit = cx.debug_bounds("titlebar-avatar").unwrap();
            cx.simulate_event(gpui::MouseMoveEvent {
                position: if hovered {
                    hit.center()
                } else {
                    point(px(100.), px(100.))
                },
                ..Default::default()
            });
            cx.update(|window, cx| {
                window.refresh();
                let _ = window.draw(cx);
            });
            let icon = cx.debug_bounds("titlebar-avatar-circle").unwrap();
            assert_eq!(icon.size, size(px(super::AVATAR), px(super::AVATAR)));
            assert_eq!(icon.center(), hit.center());
            assert_eq!(hit.size, size(px(28.), px(28.)));
        }
    }
}

#[gpui::test]
fn profile_slot_bounds_and_context_menu_do_not_start_auth(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(header_window);
    for (width, height) in [(1200., 780.), (640., 400.), (360., 400.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.run_until_parked();
        cx.update(|window, cx| {
            window.refresh();
            let _ = window.draw(cx);
        });
        let menu_bottom = cx
            .debug_bounds("application-menu-bar")
            .map_or(px(0.), |bar| bar.bottom());
        assert_eq!(
            cx.debug_bounds("titlebar").unwrap(),
            Bounds::new(point(px(0.), menu_bottom), size(px(width), px(34.)))
        );
        assert_eq!(
            cx.debug_bounds("titlebar-avatar").unwrap(),
            Bounds::new(
                point(px(width - 34.), menu_bottom + px(3.)),
                size(px(28.), px(28.))
            )
        );
        // A worktree build's banner sits at the window's foot, not here.
        assert_eq!(
            cx.debug_bounds("window-body").unwrap().top(),
            menu_bottom + px(34.)
        );
    }
    let bounds = cx.debug_bounds("titlebar-avatar").unwrap();
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Right,
        position: bounds.center(),
        modifiers: Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    cx.update(|_, cx| {
        let view = view.read(cx);
        assert!(view.menu.page == Some(Page::Profile));
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.dismiss_menu(window, cx);
            view.open_profile(window, cx);
            assert_eq!(view.menu.page, Some(Page::Profile));
        })
    });
}

#[gpui::test]
fn left_click_opens_profile_without_starting_github_sign_in(cx: &mut TestAppContext) {
    // Keep a sign-in call out of this click route before exercising it: a
    // runner may have a real OAuth client ID in its environment.
    assert!(!include_str!("../titlebar.rs").contains("start_github("));
    let (view, cx) = cx.add_window_view(header_window);
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    let hit = cx.debug_bounds("titlebar-avatar").unwrap();
    cx.simulate_click(hit.center(), Modifiers::default());
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(Page::Profile));
    });
}
