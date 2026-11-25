// ABOUTME: Exercises the shared navigation predicates every client reads a snapshot with.
// ABOUTME: Pins which destination is missing, and the focused chain a staged route needs.
use super::*;
use crate::navigation::{Missing, NavigationTarget};

fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(SNAPSHOT).unwrap()
}

#[test]
fn a_destination_exists_until_its_snapshot_boot_changes() {
    let snapshot = snapshot();
    for target in [
        NavigationTarget::Workspace("w1"),
        NavigationTarget::Tab("w1:t1"),
        NavigationTarget::Pane("w1:p1"),
    ] {
        assert_eq!(target.exists_in(&snapshot, "boot-v1"), Ok(()));
        assert_eq!(
            target.exists_in(&snapshot, "boot-v2"),
            Err(Missing::Session)
        );
        assert_eq!(target.exists_in(&snapshot, ""), Err(Missing::Session));
    }
}

#[test]
fn a_missing_destination_is_named_by_its_own_level() {
    let snapshot = snapshot();
    assert_eq!(
        NavigationTarget::Workspace("gone").exists_in(&snapshot, "boot-v1"),
        Err(Missing::Workspace)
    );
    assert_eq!(
        NavigationTarget::Tab("gone").exists_in(&snapshot, "boot-v1"),
        Err(Missing::Tab)
    );
    assert_eq!(
        NavigationTarget::Pane("gone").exists_in(&snapshot, "boot-v1"),
        Err(Missing::Pane)
    );
}

#[test]
fn a_tab_or_pane_is_reachable_only_in_the_focused_chain() {
    let mut snapshot = snapshot();
    assert!(NavigationTarget::Tab("w1:t1").reachable_in(&snapshot));
    assert!(NavigationTarget::Pane("w1:p1").reachable_in(&snapshot));
    // The same tab and pane, once the snapshot focuses elsewhere.
    snapshot.focused_workspace_id = Some("w2".into());
    snapshot.focused_tab_id = Some("w2:t1".into());
    assert!(!NavigationTarget::Tab("w1:t1").reachable_in(&snapshot));
    assert!(!NavigationTarget::Pane("w1:p1").reachable_in(&snapshot));
    // A workspace is reachable wherever it sits: focusing it is the step that
    // brings its tabs and panes into the chain.
    assert!(NavigationTarget::Workspace("w1").reachable_in(&snapshot));
}

#[test]
fn a_target_is_focused_only_at_its_own_level() {
    let snapshot = snapshot();
    assert!(NavigationTarget::Workspace("w1").is_focused_in(&snapshot));
    assert!(NavigationTarget::Tab("w1:t1").is_focused_in(&snapshot));
    assert!(NavigationTarget::Pane("w1:p1").is_focused_in(&snapshot));
    assert!(!NavigationTarget::Pane("w1:t1").is_focused_in(&snapshot));
    assert!(!NavigationTarget::Workspace("w1:p1").is_focused_in(&snapshot));
}
