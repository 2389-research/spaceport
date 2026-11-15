// ABOUTME: Pins the footer menu's rows: their order, selectors and labels.
// ABOUTME: Guards against a row that shows its raw key or does nothing.
//! The in-window app menu's rows. Their order is what users see, and their
//! `menu-{selector}` debug selectors are what tests and native checks find
//! them by, so both are pinned here.
#![allow(clippy::unwrap_used)]

use super::chrome::Item;
use crate::{
    SheprdWindow,
    i18n::leak_tests::{draw, under_en_xa},
    sidebar::layout_tests::fixture_window,
};
use gpui::{px, size};
use sheprd_i18n::pseudo::is_marked;
use std::collections::HashSet;

/// The rows a window that has never connected offers, top to bottom.
const DEFAULT_ROWS: &[&str] = &[
    "settings",
    "shortcuts",
    "themes",
    "increase font size",
    "decrease font size",
    "reset font size",
    "command palette",
    "workspaces",
    "reload GUI config",
    "about",
    "reconnect",
];

fn selectors(view: &SheprdWindow) -> Vec<&'static str> {
    view.menu_items().into_iter().map(Item::selector).collect()
}

#[gpui::test]
fn the_default_menu_lists_its_rows_in_order(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    assert_eq!(view.read_with(cx, |view, _| selectors(view)), DEFAULT_ROWS);

    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.open_menu(window, cx));
        window.draw(cx).clear(cx);
    });
    for row in DEFAULT_ROWS {
        let selector: &'static str = format!("menu-{row}").leak();
        assert!(cx.debug_bounds(selector).is_some(), "{selector} not drawn");
    }
}

/// English labels happen to spell the same as the rows' keys, so a row that
/// fell back to its key is only visible under the accented pseudo-locale.
#[test]
fn every_offered_row_draws_a_translated_label_not_its_key() {
    under_en_xa("footer menu labels", |cx, recorder| {
        let (view, cx) = cx.add_window_view(fixture_window);
        cx.simulate_resize(size(px(1000.), px(800.)));
        draw(cx);
        recorder.take();

        let rows = view.read_with(cx, |view, _| selectors(view));
        cx.update(|window, cx| view.update(cx, |view, cx| view.open_menu(window, cx)));
        draw(cx);
        let shown = recorder.take();

        for row in &rows {
            assert!(
                !shown.iter().any(|line| line.trim() == *row),
                "{row} is drawn as its raw key: {shown:#?}"
            );
        }
        let labels = shown
            .iter()
            .filter(|line| !line.trim().is_empty() && is_marked(line))
            .count();
        assert!(
            labels >= rows.len(),
            "{} rows drew {labels} translated labels: {shown:#?}",
            rows.len()
        );
    });
}

/// The rows offered only while connected, or when the daemon has news.
const CONDITIONAL_ROWS: &[&str] = &[
    "reload daemon config",
    "update ready",
    "what's new",
    "detach",
];

#[test]
fn every_item_has_its_own_pinned_selector() {
    let selectors: Vec<_> = Item::ALL.iter().map(|item| item.selector()).collect();
    let distinct: HashSet<_> = selectors.iter().copied().collect();
    assert_eq!(distinct.len(), selectors.len(), "{selectors:?}");
    let pinned: HashSet<_> = DEFAULT_ROWS
        .iter()
        .chain(CONDITIONAL_ROWS)
        .copied()
        .collect();
    assert_eq!(distinct, pinned);
}

/// Each item's own label, including the rows only a connected window or a
/// daemon with news offers.
#[test]
fn every_item_has_a_translated_label_under_en_xa() {
    under_en_xa("footer menu item labels", |_, _| {
        for item in Item::ALL {
            let label = item.label();
            assert!(!label.trim().is_empty(), "{item:?} has no label");
            assert!(is_marked(&label), "{item:?} shows {label:?}");
            assert_ne!(label, item.selector(), "{item:?} shows its raw key");
        }
    });
}
