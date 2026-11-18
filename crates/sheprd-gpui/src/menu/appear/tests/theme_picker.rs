// ABOUTME: Exercises picker departure with native input ownership and scroll geometry.
// ABOUTME: Requires immediate input cleanup without disturbing retained write ownership.
use super::*;
use crate::theme_picker::tests::departure::fixture;
use gpui::{Entity, ScrollDelta, ScrollEvent, TestAppContext, VisualTestContext};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn close(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().expect("picker leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        });
    });
}

#[gpui::test]
fn closing_themes_releases_the_search_and_reopening_creates_another(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture);
    let old = cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_theme_picker(window, cx);
            view.menu
                .themes
                .as_ref()
                .expect("open picker")
                .search()
                .expect("open search")
                .downgrade()
        })
    });
    draw(cx);
    close(&view, cx);
    draw(cx);
    assert!(
        old.upgrade().is_none(),
        "dismissal releases its input owner"
    );
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_theme_picker(window, cx)));
    assert!(
        old.upgrade().is_none(),
        "a reopened picker gets a fresh field"
    );
    draw(cx);
    assert!(cx.debug_bounds("theme-row-0").is_some());
}

#[gpui::test]
fn themes_keep_scrolled_rows_after_live_results_change(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_theme_picker(window, cx);
        });
    });
    draw(cx);
    let row = cx
        .debug_bounds("theme-row-5")
        .expect("visible built-in row");
    cx.simulate_event(ScrollEvent {
        position: row.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("theme-row-5").expect("scrolled row");
    assert!(scrolled.top() < row.top(), "the live list actually scrolls");
    close(&view, cx);
    view.update(cx, |view, _| {
        view.menu
            .themes
            .as_mut()
            .expect("retained lifecycle owner")
            .filtered
            .clear();
    });
    draw(cx);
    let leaving = cx
        .debug_bounds("theme-row-5")
        .expect("captured departing row");
    assert!((leaving.top() - scrolled.top()).abs() < px(1.));
}
