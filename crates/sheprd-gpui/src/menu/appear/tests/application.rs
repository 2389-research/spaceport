// ABOUTME: Checks custom application-menu departure after action and focus owners reset.
// ABOUTME: Preserves visible panel content while the underlying window regains input.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{menu::Cover, sidebar::layout_tests::fixture_window};
use gpui::{Entity, TestAppContext, VisualTestContext};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

#[gpui::test]
fn custom_application_menu_leaves_after_its_active_owner_resets(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        crate::bind_keys(cx);
        let view = fixture_window(window, cx);
        window.focus(&view.focus, cx);
        view
    });
    draw(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_application_menu(Some(0), window, cx)
        })
    });
    draw(cx);
    assert!(cx.debug_bounds("application-menu-panel").is_some());
    assert!(
        cx.debug_bounds("application-item-About Spaceport")
            .is_some()
    );
    close(&view, cx);
    draw(cx);
    assert!(cx.debug_bounds("application-menu-panel").is_some());
    assert!(
        cx.debug_bounds("application-item-About Spaceport")
            .is_some()
    );
    cx.update(|window, cx| {
        let view = view.read(cx);
        assert!(view.menu.application.is_none());
        assert!(view.menu.page.is_none());
        assert!(view.focus.is_focused(window));
        assert_eq!(view.menu.cover.get(), Cover::Unknown);
    });
}

fn close(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            if let Some(leaving) = &mut view.menu.appear.leaving {
                leaving.motion.set(true, later, true);
                leaving.motion.set(false, later, false);
            }
        });
    });
}

#[gpui::test]
fn menu_invalidation_keeps_only_departure_paint_and_releases_its_cover(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        crate::bind_keys(cx);
        let view = fixture_window(window, cx);
        window.focus(&view.focus, cx);
        view
    });
    draw(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_application_menu(Some(0), window, cx)
        })
    });
    draw(cx);
    view.update(cx, |view, cx| {
        view.menu.appear.live = true;
        view.close_stale_application_menu(cx);
        let later = motion::now() + std::time::Duration::from_secs(3600);
        view.menu.appear.observe(None, None, later);
        let leaving = view.menu.appear.leaving.as_mut().unwrap();
        leaving.motion.set(true, later, true);
        leaving.motion.set(false, later, false);
    });
    draw(cx);
    assert!(
        cx.debug_bounds("application-item-About Spaceport")
            .is_some()
    );
    cx.update(|window, cx| {
        let view = view.read(cx);
        assert!(view.menu.application.is_none());
        assert!(view.menu.page.is_none());
        assert!(view.focus.is_focused(window));
        assert_eq!(view.menu.cover.get(), Cover::Unknown);
    });
}
