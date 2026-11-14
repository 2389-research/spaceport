// ABOUTME: Verifies Ask startup gates helper ownership and activity visibility.
// ABOUTME: Keeps preference edits pending until a later application launch.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{ask, sidebar::ActivityPanel};

#[gpui::test]
fn disabled_startup_never_creates_a_worker_or_restores_a_request(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        ask::install(false, true, cx);
        assert!(!cx.has_global::<ask::Shared>());
        let owner = ask::controller(cx);
        let owner = owner.read(cx);
        assert!(owner.worker.is_none() && owner.link.is_none() && owner.pending.is_none());
        assert!(!owner.active && !owner.loading);
        assert!(ask::status(cx).is_none());
    });
}

#[gpui::test]
fn saved_edits_do_not_change_existing_or_new_windows(cx: &mut gpui::TestAppContext) {
    for enabled in [false, true] {
        cx.update(|cx| Feature::install(enabled, cx));
        let first = cx.add_window(crate::sidebar::layout_tests::fixture_window);
        let second = cx.add_window(crate::sidebar::layout_tests::fixture_window);
        for window in [first, second] {
            cx.update(|cx| {
                window
                    .update(cx, |view, _, cx| {
                        view.config.chat.ask_enabled = !enabled;
                        assert!(Feature::restart_required(!enabled, cx));
                        assert_eq!(view.activity_panel_listed(ActivityPanel::Ask), enabled);
                        assert!(!Feature::restart_required(enabled, cx));
                    })
                    .unwrap();
            });
        }
    }
}

#[gpui::test]
fn disabled_saved_panel_falls_back_to_workspaces(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.config.sidebar_style.activity_bar = true;
            view.activity.panel = ActivityPanel::Ask;
            assert!(view.activity_section().is_none());
            view.show_activity_panel(ActivityPanel::Ask, window, cx);
            assert!(view.activity_section().is_none());
            assert!(!view.ask_feature.enabled());
            assert!(!cx.has_global::<ask::Shared>());
        });
    });
}
