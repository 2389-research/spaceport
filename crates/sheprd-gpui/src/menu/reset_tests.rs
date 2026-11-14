//! What closing the menu leaves behind: a page's own entities and
//! subscriptions must go with the page, whichever path reset the menu.

#![allow(clippy::unwrap_used)]

use super::Page;
use crate::{
    SheprdWindow,
    search_input::SearchInput,
    sidebar::layout_tests::{fixture_window, full_draw},
};
use gpui::{Entity, TestAppContext, VisualTestContext, WeakEntity, px, size};

/// A window showing the keyboard shortcuts page with `query` typed into its
/// search, and a weak handle that tells whether the search still exists.
fn shortcuts_open<'a>(
    cx: &'a mut TestAppContext,
    query: &str,
) -> (
    Entity<SheprdWindow>,
    WeakEntity<SearchInput>,
    &'a mut VisualTestContext,
) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.simulate_resize(size(px(700.), px(600.)));
    let search = view.update_in(cx, |view, window, cx| {
        view.open_keybinds(window, cx);
        assert_eq!(view.menu.page, Some(Page::Keybinds));
        assert!(view.menu._keybinds_subscription.is_some());
        view.menu.keybinds_search.as_ref().unwrap().downgrade()
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    cx.simulate_input(query);
    search
        .read_with(cx, |search, _| assert_eq!(search.text(), query))
        .unwrap();
    (view, search, cx)
}

#[gpui::test]
fn dismissing_the_shortcuts_page_drops_its_search(cx: &mut TestAppContext) {
    let (view, search, cx) = shortcuts_open(cx, "zoom");

    view.update_in(cx, |view, window, cx| view.dismiss_menu(window, cx));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));

    view.read_with(cx, |view, _| {
        assert!(view.menu.page.is_none());
        assert!(
            view.menu.keybinds_search.is_none(),
            "the closed page's search field is still held"
        );
        assert!(
            view.menu._keybinds_subscription.is_none(),
            "the closed page's search subscription is still held"
        );
    });
    assert!(
        search.upgrade().is_none(),
        "the search entity outlived its page"
    );
}

/// A detach or a retarget resets the menu without going through
/// `dismiss_menu`, so nothing moves the focus off the search field.
#[gpui::test]
fn resetting_under_the_shortcuts_page_drops_its_search(cx: &mut TestAppContext) {
    let (view, search, cx) = shortcuts_open(cx, "zoom");

    view.update(cx, |view, cx| {
        view.reset_selected(cx);
        cx.notify();
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    // Typed after the page closed: no field of that page may take it.
    cx.simulate_input("x");

    // What the page's search holds now, if anything still keeps it.
    let held = search
        .upgrade()
        .map(|search| search.read_with(cx, |search, _| search.text().to_owned()));
    assert_eq!(held, None, "the search entity outlived its page");
    view.read_with(cx, |view, _| {
        assert!(view.menu.page.is_none());
        assert!(view.menu.keybinds_search.is_none());
        assert!(view.menu._keybinds_subscription.is_none());
    });
}
