// ABOUTME: Keeps a closing usage panel tied to its captured observation.
// ABOUTME: Uses synthetic reports instead of provider feeds or private account data.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{
    sidebar::layout_tests::fixture_window,
    usage::{
        Host, Provider, Reading, Usage,
        model::{Account, Kind, Report, Window as Limit},
    },
};
use gpui::{TestAppContext, VisualTestContext};

fn reports(ids: &[&str]) -> Usage {
    report_rows(ids, 1)
}

fn report_rows(ids: &[&str], rows: usize) -> Usage {
    Usage::showing(
        Host::Local,
        ids.iter()
            .map(|id| {
                let provider = sheprd_usage::registry::find(id).unwrap();
                Reading {
                    provider,
                    report: Some(Report::new(
                        provider,
                        Account::default(),
                        (0..rows)
                            .map(|_| Limit::new(Kind::Session, 25., None, None))
                            .collect(),
                    )),
                    error: None,
                }
            })
            .collect(),
    )
}

#[gpui::test]
fn departing_usage_content_keeps_its_real_scroll_position(cx: &mut TestAppContext) {
    let provider = sheprd_usage::registry::find("claude").unwrap();
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.usage = report_rows(&["claude", "codex"], 12);
        view
    });
    cx.simulate_resize(gpui::size(px(500.), px(360.)));
    draw(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_usage(provider, point(px(300.), px(340.)), window, cx)
        })
    });
    draw(cx);
    let panel = cx.debug_bounds("usage-panel").unwrap();
    let before = cx.debug_bounds("usage-panel-header").unwrap();
    cx.simulate_event(gpui::ScrollEvent {
        position: panel.center(),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-20.))),
        ..Default::default()
    });
    draw(cx);
    let scrolled = cx.debug_bounds("usage-panel-header").unwrap();
    assert!(scrolled.top() < before.top());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().unwrap();
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        })
    });
    draw(cx);
    assert_eq!(cx.debug_bounds("usage-panel").unwrap(), panel);
    assert_eq!(cx.debug_bounds("usage-panel-header").unwrap(), scrolled);
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
}

#[gpui::test]
fn a_departing_usage_panel_does_not_follow_a_new_observation(cx: &mut TestAppContext) {
    let provider: Provider = sheprd_usage::registry::find("claude").unwrap();
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.config.usage.show = true;
        view.usage = reports(&["claude", "codex"]);
        view
    });
    draw(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_usage(provider, point(px(300.), px(500.)), window, cx)
        })
    });
    draw(cx);
    assert!(cx.debug_bounds("usage-tab-codex").is_some());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view.menu.appear.leaving.as_mut().unwrap();
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
            view.usage = reports(&["grok", "cursor"]);
        })
    });
    draw(cx);
    assert!(cx.debug_bounds("usage-tab-codex").is_some());
    assert!(cx.debug_bounds("usage-tab-cursor").is_none());
    view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
}
