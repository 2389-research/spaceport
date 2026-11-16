// ABOUTME: Checks the card over Local when no Herdr is installed on this computer.
// ABOUTME: It replaces the reconnecting card, links the website, and retries.
use super::*;
use crate::i18n::fl;
use gpui::{Element, IntoElement, Modifiers, accesskit};

const MISSING: &str =
    "client I/O failed: Could not start herdr server: No such file or directory (os error 2)";

#[gpui::test]
fn a_missing_herdr_shows_the_install_card_not_the_reconnecting_card(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        view.endpoints[0].never_connect();
        view.endpoints[0].lose_herdr_for_test(MISSING);
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("herdr-missing").is_some());
    assert!(cx.debug_bounds("reconnect-now").is_none());
    assert!(cx.debug_bounds("reconnecting-reason").is_none());

    let install = cx.debug_bounds("install-herdr").unwrap();
    cx.simulate_click(install.center(), Modifiers::default());
    assert_eq!(cx.opened_url().as_deref(), Some(crate::about::WEBSITE));

    let generation = view.read_with(cx, |view, _| view.endpoints[0].generation);
    let again = cx.debug_bounds("herdr-try-again").unwrap();
    cx.simulate_click(again.center(), Modifiers::default());
    view.read_with(cx, |view, _| {
        assert!(
            view.endpoints[0].generation > generation,
            "Try again redials as Reconnect now does"
        );
    });
}

#[gpui::test]
fn an_ordinary_outage_keeps_the_reconnecting_card(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        view.endpoints[0].never_connect();
        view.endpoints[0].lose_connection_for_test("connection lost");
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("reconnect-now").is_some());
    assert!(cx.debug_bounds("herdr-missing").is_none());
}

#[test]
fn the_cards_buttons_are_named_buttons() {
    let theme = crate::config::Theme::default();
    for label in [
        fl!("window-install-herdr"),
        fl!("window-try-again"),
        fl!("window-reconnect-now"),
    ] {
        let element =
            super::super::reconnecting::card_button("button", label.clone(), &theme).into_element();
        let mut node = accesskit::Node::new(element.a11y_role().unwrap());
        element.write_a11y_info(&mut node);
        assert_eq!(node.role(), accesskit::Role::Button);
        assert_eq!(node.label(), Some(label.as_str()));
    }
}
