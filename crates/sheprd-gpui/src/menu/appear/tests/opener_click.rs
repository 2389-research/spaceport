// ABOUTME: Keeps live popup openers clickable while closed content departs.
// ABOUTME: Reproduces native Shortcuts dismissal without wall-clock waits.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::sidebar::layout_tests::{fixture_window, full_draw};
use gpui::{Modifiers, TestAppContext, VisualTestContext, size};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

#[gpui::test]
fn departing_shortcuts_do_not_block_the_live_sidebar_menu_opener(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(size(px(800.), px(854.)));
    draw(cx);
    let opener = cx.debug_bounds("sidebar-menu").unwrap().center();
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_keybinds(window, cx));
    });
    draw(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().unwrap();
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
            assert!(view.menu.page.is_none());
            assert!(view.menu.keybinds_search.is_none());
        });
    });
    draw(cx);
    let outgoing = cx.debug_bounds("menu-panel").unwrap();
    assert!(
        outgoing.contains(&opener),
        "fixture must reproduce overlapping bounds: {outgoing:?} {opener:?}"
    );
    cx.simulate_click(opener, Modifiers::default());
    assert_eq!(
        view.read_with(cx, |view, _| view.menu.page),
        Some(Page::Menu)
    );
}
