// ABOUTME: Verifies the experimental Core opt-in and its restart-only settings UI.
// ABOUTME: Keeps disabled Core controls hidden while allowing the saved toggle to change.
use super::*;
use crate::config::preferences::Preference;
use crate::settings_window::catalog;
use crate::settings_window::controls::preferences::PreferenceIo;
use std::sync::{Arc, Mutex};

#[gpui::test]
fn disabled_core_hides_the_legacy_tabs_setting(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    view.update(cx, |view, cx| {
        view.section = Section::Tabs;
        cx.notify();
    });
    cx.simulate_resize(size(px(960.), px(2200.)));
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("tabs-layout").is_some());
    assert!(cx.debug_bounds("tabs-herdr-2-toggle").is_none());
}

#[gpui::test]
fn core_opt_in_saves_and_shows_restart_without_changing_the_running_mode(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    let file = Arc::new(Mutex::new(false));
    let (written, read) = (file.clone(), file.clone());
    view.update(cx, |view, cx| {
        view.controls.preference_io = Some(PreferenceIo::with_load(
            move |edit| {
                let Preference::HerdrCoreEnabled(enabled) = edit else {
                    panic!("unexpected preference: {edit:?}");
                };
                *written.lock().unwrap() = enabled;
            },
            move || {
                let mut loaded = skill_load()?;
                loaded.config.herdr_core_enabled = *read.lock().unwrap();
                Ok(loaded)
            },
        ));
        view.section = Section::General;
        cx.notify();
    });
    cx.simulate_resize(size(px(960.), px(3200.)));
    for saved in [true, false] {
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        let switch = cx.debug_bounds("general-herdr-core-toggle").unwrap();
        cx.simulate_click(switch.center(), Default::default());
        cx.run_until_parked();
        assert_eq!(*file.lock().unwrap(), saved);
        view.read_with(cx, |view, cx| {
            assert_eq!(view.config.herdr_core_enabled, saved);
            assert!(!crate::core_feature::enabled(cx));
            assert_eq!(crate::core_feature::restart_required(saved, cx), saved);
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        assert_eq!(
            cx.debug_bounds("general-herdr-core-restart").is_some(),
            saved
        );
    }
}

#[gpui::test]
fn pending_enable_keeps_other_core_controls_and_search_results_hidden(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(skill_fixture);
    view.update(cx, |view, cx| {
        view.config.herdr_core_enabled = true;
        view.set_search_query("core", cx);
        let visible: Vec<_> = view
            .visible_results(cx)
            .iter()
            .map(|index| catalog::CATALOG[*index].id)
            .collect();
        assert!(visible.contains(&"general-herdr-core"));
        assert!(!visible.contains(&"tabs-herdr-2"));
        assert!(!visible.contains(&"sidebar-automations"));
        view.clear_search(cx);
    });
    cx.simulate_resize(size(px(960.), px(2200.)));
    for (section, hidden) in [
        (Section::Tabs, "tabs-herdr-2-toggle"),
        (Section::Sidebar, "sidebar-automations-toggle"),
    ] {
        view.update(cx, |view, cx| {
            view.section = section;
            cx.notify();
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        assert!(cx.debug_bounds(hidden).is_none());
    }
}

#[gpui::test]
fn pending_disable_preserves_core_controls_until_restart(cx: &mut TestAppContext) {
    cx.update(|cx| crate::core_feature::install(true, cx));
    let (view, cx) = cx.add_window_view(skill_fixture);
    cx.simulate_resize(size(px(960.), px(2200.)));
    for (section, shown) in [
        (Section::Tabs, "tabs-herdr-2-toggle"),
        (Section::Sidebar, "sidebar-automations-toggle"),
    ] {
        view.update(cx, |view, cx| {
            assert!(!view.config.herdr_core_enabled);
            assert!(crate::core_feature::restart_required(false, cx));
            view.section = section;
            cx.notify();
        });
        cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
        assert!(cx.debug_bounds(shown).is_some());
    }
    view.update(cx, |view, cx| {
        view.set_search_query("core", cx);
        let visible: Vec<_> = view
            .visible_results(cx)
            .iter()
            .map(|index| catalog::CATALOG[*index].id)
            .collect();
        for shown in ["general-herdr-core", "tabs-herdr-2", "sidebar-automations"] {
            assert!(visible.contains(&shown), "missing search result: {shown}");
        }
    });
}
