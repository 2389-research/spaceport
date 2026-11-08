use super::*;
use gpui::{px, size};

#[test]
fn settings_catalog_excludes_legacy_plugin_values_but_keeps_integrations() {
    assert!(
        Section::ALL
            .iter()
            .all(|section| section.message() != "settings-section-plugins")
    );
    assert!(
        catalog::CATALOG
            .iter()
            .all(|entry| entry.id != "plugins-values")
    );
    assert!(Section::ALL.contains(&Section::Integrations));
    assert!(
        catalog::CATALOG
            .iter()
            .any(|entry| entry.id == "integrations-search")
    );
}

#[gpui::test]
fn settings_navigation_and_search_do_not_offer_plugin_values(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.simulate_resize(size(px(960.), px(900.)));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("settings-section-plugins").is_none());
    assert!(cx.debug_bounds("settings-section-integrations").is_some());

    view.update(cx, |view, cx| view.set_search_query("plugin values", cx));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("settings-search-empty").is_some());
    assert!(cx.debug_bounds("settings-search-result-0").is_none());
}
