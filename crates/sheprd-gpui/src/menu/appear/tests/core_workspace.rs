// ABOUTME: Preserves closing Core workspace forms across observation changes and scrolling.
// ABOUTME: Uses synthetic documents without starting or editing a Core runtime.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::{core_terminals, sidebar::layout_tests::fixture_core_window};
use gpui::{ScrollDelta, ScrollEvent, TestAppContext};
use sheprd_core_client::wire::Doc;
use uuid::Uuid;

fn observe(cx: &mut gpui::App, present: bool) {
    core_terminals::set_document_for_test_as(
        cx,
        "fixture-core",
        Doc {
            value: serde_json::json!({"schema_version":1,"workspaces": if present {
            serde_json::json!([{"id":Uuid::from_u128(71),"label":"Original group"}])
        } else { serde_json::json!([]) }}),
            version: if present { 3 } else { 4 },
        },
    );
}

fn open(view: &mut SheprdWindow, page: Page, window: &mut Window, cx: &mut Context<SheprdWindow>) {
    match page {
        Page::CoreWorkspaceActions => view.open_core_workspace_actions(
            Uuid::from_u128(71),
            point(px(20.), px(100.)),
            window,
            cx,
        ),
        Page::CoreWorkspaceCreate => view.open_core_workspace_dialog(window, cx),
        Page::CoreWorkspaceRename => {
            view.open_core_workspace_rename(Uuid::from_u128(71), window, cx)
        }
        Page::CoreWorkspaceDelete => {
            view.open_core_workspace_delete(Uuid::from_u128(71), window, cx)
        }
        _ => unreachable!(),
    }
}

fn close(view: &mut SheprdWindow, window: &mut Window, cx: &mut Context<SheprdWindow>) {
    view.menu.appear.live = true;
    view.dismiss_menu(window, cx);
    let later = motion::now() + std::time::Duration::from_secs(3600);
    view.menu.appear.observe(None, None, later);
    let leaving = view.menu.appear.leaving.as_mut().expect("Core form leaves");
    leaving.motion.set(true, later, true);
    leaving.motion.set(false, later, false);
}

#[gpui::test]
fn core_workspace_forms_keep_their_content_after_observation_changes(cx: &mut TestAppContext) {
    let mut changed = Vec::new();
    for (page, selector) in [
        (Page::CoreWorkspaceActions, "core-workspace-menu-name"),
        (Page::CoreWorkspaceCreate, "dialog-input"),
        (Page::CoreWorkspaceRename, "dialog-input"),
        (Page::CoreWorkspaceDelete, "core-workspace-delete-summary"),
    ] {
        let (view, cx) = cx.add_window_view(fixture_core_window);
        cx.update(|window, cx| {
            observe(cx, true);
            view.update(cx, |view, cx| open(view, page, window, cx));
            window.draw(cx).clear(cx);
        });
        let before = cx.debug_bounds(selector).expect("live Core form");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| close(view, window, cx));
            observe(cx, false);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        if cx.debug_bounds(selector) != Some(before) {
            changed.push((page, before, cx.debug_bounds(selector)));
        }
    }
    assert!(
        changed.is_empty(),
        "Core departure content changed: {changed:?}"
    );
}

#[gpui::test]
fn core_workspace_forms_keep_their_shell_scroll_when_leaving(cx: &mut TestAppContext) {
    let mut changed = Vec::new();
    for page in [Page::CoreWorkspaceActions, Page::CoreWorkspaceCreate] {
        let (view, cx) = cx.add_window_view(fixture_core_window);
        cx.simulate_resize(gpui::size(px(320.), px(240.)));
        cx.update(|window, cx| {
            observe(cx, true);
            view.update(cx, |view, cx| {
                open(view, page, window, cx);
                view.menu.error = Some("Synthetic Core document error. ".repeat(80));
            });
            window.draw(cx).clear(cx);
        });
        let selector = if page == Page::CoreWorkspaceActions {
            "core-workspace-menu-name"
        } else {
            "core-workspace-dialog-title"
        };
        let before = cx.debug_bounds(selector).expect("live form");
        let panel = cx.debug_bounds("menu-panel").expect("Core shell");
        cx.simulate_event(ScrollEvent {
            position: panel.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(-80.))),
            ..Default::default()
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let scrolled = cx.debug_bounds(selector).expect("scrolled Core form");
        assert!(scrolled.top() < before.top(), "Core shell scrolls");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| close(view, window, cx));
            window.refresh();
            window.draw(cx).clear(cx);
        });
        if cx.debug_bounds(selector) != Some(scrolled) {
            changed.push((page, scrolled, cx.debug_bounds(selector)));
        }
    }
    assert!(
        changed.is_empty(),
        "Core departure scroll changed: {changed:?}"
    );
}
