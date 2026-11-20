// ABOUTME: Checks exact workspace groups and stale consent against real wire snapshots.
// ABOUTME: Prevents group or pane membership changes from widening a confirmed close.
#![allow(clippy::unwrap_used)]
use super::*;
use sheprd_client::protocol::ClientShellWorktree;

fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../../../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}

#[test]
fn a_single_workspace_requests_only_its_exact_id() {
    let snapshot = snapshot();
    let id = &snapshot.workspaces[0].workspace_id;
    let target = Target::capture(&snapshot, id).unwrap();
    let (method, params) = target.request(&snapshot).unwrap();
    assert_eq!(method, Method::WorkspaceClose);
    assert_eq!(params, json!({"workspace_id": id, "close_group": false}));
}

#[test]
fn changed_boot_label_tabs_or_panes_reject_the_captured_close() {
    let original = snapshot();
    let id = &original.workspaces[0].workspace_id;
    let target = Target::capture(&original, id).unwrap();
    for change in 0..4 {
        let mut current = original.clone();
        match change {
            0 => current.boot_id.push('x'),
            1 => current.workspaces[0].label.push('x'),
            2 => current.tabs[0].tab_id.push('x'),
            _ => current.panes[0].pane_id.push('x'),
        }
        assert!(matches!(
            target.request(&current),
            Err(Error::StaleCloseTarget)
        ));
    }
}

#[test]
fn primary_groups_are_explicit_and_new_members_invalidate_consent() {
    let mut snapshot = snapshot();
    let id = snapshot.workspaces[0].workspace_id.clone();
    snapshot.workspaces[0].worktree = Some(ClientShellWorktree {
        key: "fixture-repo".into(),
        label: "main".into(),
        is_linked_worktree: false,
    });
    let mut linked = snapshot.workspaces[0].clone();
    linked.workspace_id = "linked".into();
    linked.worktree.as_mut().unwrap().is_linked_worktree = true;
    snapshot.workspaces.push(linked.clone());
    let target = Target::capture(&snapshot, &id).unwrap();
    assert_eq!(target.workspace_count(), 2);
    assert_eq!(target.request(&snapshot).unwrap().1["close_group"], true);
    assert_eq!(
        Target::capture(&snapshot, "linked")
            .unwrap()
            .workspace_count(),
        1
    );
    linked.workspace_id = "another".into();
    snapshot.workspaces.push(linked);
    assert!(matches!(
        target.request(&snapshot),
        Err(Error::StaleCloseTarget)
    ));
    snapshot
        .workspaces
        .iter_mut()
        .find(|workspace| workspace.workspace_id == "another")
        .unwrap()
        .worktree
        .as_mut()
        .unwrap()
        .is_linked_worktree = false;
    assert_eq!(
        Target::capture(&snapshot, &id).unwrap().workspace_count(),
        1
    );
}
