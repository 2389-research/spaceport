// ABOUTME: Checks display filtering after provider polling no longer filters reports.
// ABOUTME: Exercises the retained status-bar view against a headless native-layout fixture.
use super::*;
use crate::sidebar::layout_tests::{fixture_window, full_draw};
use gpui::{TestAppContext, px, size};

#[gpui::test]
fn hidden_reports_stay_out_of_the_status_bar(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.status = crate::state::ConnectionStatus::Connected;
        view.config.usage.show = true;
        view.config.usage.hide_providers = vec!["claude".into()];
        view.usage = super::super::Usage::showing(
            super::super::Host::Local,
            vec![
                Reading {
                    provider: provider("claude"),
                    report: Some(report(provider("claude"), 90.)),
                    error: None,
                },
                Reading {
                    provider: provider("codex"),
                    report: Some(report(provider("codex"), 0.)),
                    error: None,
                },
            ],
        );
        view
    });
    cx.simulate_resize(size(px(1600.), px(600.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("usage-codex").is_some());
    assert!(cx.debug_bounds("usage-claude").is_none());
}
