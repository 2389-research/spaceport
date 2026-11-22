// ABOUTME: Checks local menu controls and absence of account workers.
// ABOUTME: Protects retained settings routes without authentication state.
//! The application menu keeps its general actions without exposing sign-in.
//! Local controls remain available without an account or profile shell.

use super::chrome::Item;
use crate::sidebar::layout_tests::fixture_window;

#[gpui::test]
fn application_menu_keeps_settings_and_excludes_github_sign_in(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.read_with(cx, |view, _| {
        let items = view.menu_items();
        assert!(items.contains(&Item::Settings));
        assert!(items.contains(&Item::Shortcuts));
        assert!(items.contains(&Item::Themes));
        assert!(!items.iter().any(|item| item.selector() == "GitHub sign-in"));
    });
}

#[test]
fn window_tick_and_popup_hold_no_account_worker() {
    assert!(!include_str!("../window.rs").contains("poll_github("));
    assert!(!include_str!("state.rs").contains("crate::github::Auth"));
    assert!(!include_str!("chrome.rs").contains("render_github_auth("));
}
