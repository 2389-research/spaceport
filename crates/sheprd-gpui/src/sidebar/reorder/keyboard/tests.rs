// ABOUTME: Sends workspace reorder keys through a real GPUI row and client wire.
// ABOUTME: Checks grouping, bounds, and stale callback refusal without a daemon.
#![allow(clippy::unwrap_used)]
use crate::{SheprdWindow, sidebar::layout_tests::fixture_window, window::MockPeer};
use gpui::{
    AppContext, Context, Entity, FocusHandle, IntoElement, Render, TestAppContext,
    VisualTestContext, Window, div, prelude::*,
};
use sheprd_client::workspaces::{WorkspaceMoveBlockParams, WorkspaceReorder};
use std::sync::Arc;

struct Row {
    owner: Entity<SheprdWindow>,
    focus: FocusHandle,
    workspace: String,
}
impl Render for Row {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.owner.update(cx, |view, cx| {
            let endpoint = view.endpoints[view.selected_endpoint].id.clone();
            let row = view.workspace_reorder_keys(
                div().id("reorder-row").track_focus(&self.focus),
                &endpoint,
                &self.workspace,
                cx,
            );
            SheprdWindow::shortcut_scope(cx).child(row)
        })
    }
}

fn row<'a>(
    peer: &MockPeer,
    workspace: &str,
    cx: &'a mut TestAppContext,
) -> (Entity<Row>, &'a mut VisualTestContext) {
    let (row, cx) = cx.add_window_view(|window, cx| {
        let owner = cx.new(|cx| {
            let mut view = fixture_window(window, cx);
            peer.prepare(&mut view, cx);
            let mut snapshot = view.live.snapshot.as_ref().unwrap().as_ref().clone();
            snapshot.workspaces = crate::sidebar::reorder::tests::list();
            view.live.snapshot = Some(Arc::new(snapshot));
            view
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Row {
            owner,
            focus,
            workspace: workspace.into(),
        }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    (row, cx)
}

#[gpui::test]
fn reorder_keys_move_a_group_and_keep_children_inside_it(cx: &mut TestAppContext) {
    for (workspace, keys, moved, before) in [
        ("a", "alt-down", vec!["a", "a1", "a2"], Some("c")),
        ("a1", "alt-down", vec!["a1"], Some("c")),
        ("a2", "alt-up", vec!["a2"], Some("a1")),
    ] {
        let mut peer = MockPeer::advertising(&["workspace.move", "workspace.move_block"]);
        let (_, cx) = row(&peer, workspace, cx);
        cx.simulate_keystrokes(keys);
        let request = peer.request();
        assert_eq!(request["method"], "workspace.move_block");
        assert_eq!(
            request["params"],
            serde_json::json!({"workspace_ids": moved, "before_workspace_id": before})
        );
    }
}

#[gpui::test]
fn reorder_keys_refuse_boundaries_and_other_chords(cx: &mut TestAppContext) {
    for (workspace, keys) in [
        ("a", "alt-up"),
        ("c", "alt-down"),
        ("a1", "alt-up"),
        ("a2", "alt-down"),
        ("a", "down"),
        ("a", "alt-shift-down"),
    ] {
        let mut peer = MockPeer::advertising(&["workspace.move_block"]);
        let (row, cx) = row(&peer, workspace, cx);
        cx.simulate_keystrokes(keys);
        let boot = row.read_with(cx, |row, cx| {
            row.owner
                .read(cx)
                .live
                .snapshot
                .as_ref()
                .unwrap()
                .boot_id
                .clone()
        });
        // An ordered request proves that a refused key queued no earlier reorder.
        peer.client
            .handle
            .reorder_workspaces(
                &boot,
                &WorkspaceReorder::MoveBlock(WorkspaceMoveBlockParams {
                    workspace_ids: vec!["ordered-marker".into()],
                    before_workspace_id: None,
                }),
            )
            .unwrap();
        assert_eq!(
            peer.request()["params"]["workspace_ids"],
            serde_json::json!(["ordered-marker"]),
            "{workspace} {keys}"
        );
    }
}

#[gpui::test]
fn captured_workspace_rows_refuse_changed_owners_and_open_dialogs(cx: &mut TestAppContext) {
    for changed in [
        "endpoint",
        "epoch",
        "generation",
        "boot",
        "dialog",
        "disconnected",
        "drag",
    ] {
        let mut peer = MockPeer::advertising(&["workspace.move_block"]);
        let (row, cx) = row(&peer, "a", cx);
        let boot = row.read_with(cx, |row, cx| {
            row.owner
                .read(cx)
                .live
                .snapshot
                .as_ref()
                .unwrap()
                .boot_id
                .clone()
        });
        row.update(cx, |row, cx| {
            row.owner.update(cx, |view, cx| {
                let endpoint = view.endpoints[view.selected_endpoint].id.clone();
                let target = super::RowTarget::capture(view, &endpoint, "a").unwrap();
                match changed {
                    "endpoint" => view.endpoints[view.selected_endpoint].id = "other-owner".into(),
                    "epoch" => view.selection_epoch += 1,
                    "generation" => view.endpoints[view.selected_endpoint].generation += 1,
                    "boot" => {
                        Arc::make_mut(view.live.snapshot.as_mut().unwrap()).boot_id =
                            "new-boot".into()
                    }
                    "dialog" => view.menu.page = Some(crate::menu::Page::Menu),
                    "disconnected" => {
                        view.live.status = crate::state::ConnectionStatus::Disconnected
                    }
                    "drag" => {
                        view.press_workspace("a", gpui::point(gpui::px(0.), gpui::px(0.)), cx)
                    }
                    _ => unreachable!(),
                }
                view.move_workspace_row(&target, true, cx);
            })
        });
        peer.client
            .handle
            .reorder_workspaces(
                &boot,
                &WorkspaceReorder::MoveBlock(WorkspaceMoveBlockParams {
                    workspace_ids: vec!["ordered-marker".into()],
                    before_workspace_id: None,
                }),
            )
            .unwrap();
        assert_eq!(
            peer.request()["params"]["workspace_ids"],
            serde_json::json!(["ordered-marker"]),
            "{changed}"
        );
    }
}

#[gpui::test]
fn configured_commands_keep_priority_over_row_reorder_keys(cx: &mut TestAppContext) {
    let mut peer = MockPeer::advertising(&["workspace.move_block"]);
    let (row, cx) = row(&peer, "a", cx);
    let (boot, before) = row.update(cx, |row, cx| {
        row.owner.update(cx, |view, _| {
            view.keymap = crate::keymap::Keymap::with_overrides(
                &[(
                    "toggle_sidebar".into(),
                    crate::keymap::Binding::One("alt-down".into()),
                )]
                .into(),
                &Default::default(),
                &crate::keymap::DaemonKeys::default(),
            )
            .unwrap();
            (
                view.live.snapshot.as_ref().unwrap().boot_id.clone(),
                view.sidebar_visible,
            )
        })
    });
    cx.update(|window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    });
    cx.simulate_keystrokes("alt-down");
    assert_ne!(
        row.read_with(cx, |row, cx| row.owner.read(cx).sidebar_visible),
        before
    );
    peer.client
        .handle
        .reorder_workspaces(
            &boot,
            &WorkspaceReorder::MoveBlock(WorkspaceMoveBlockParams {
                workspace_ids: vec!["ordered-marker".into()],
                before_workspace_id: None,
            }),
        )
        .unwrap();
    assert_eq!(
        peer.request()["params"]["workspace_ids"],
        serde_json::json!(["ordered-marker"])
    );
}
