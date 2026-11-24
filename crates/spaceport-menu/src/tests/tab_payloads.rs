use super::*;
use crate::{
    Error,
    tab::{Action, Offers, Target},
};
use spaceport_client::protocol::ClientShellPane;

/// The snapshot with `ids` as the panes of the tab this menu captures.
fn snapshot_with_tab_panes(ids: &[&str]) -> ClientShellSnapshot {
    let mut snapshot = snapshot();
    snapshot.panes.retain(|pane| pane.tab_id != "t1");
    snapshot
        .panes
        .extend(ids.iter().enumerate().map(|(at, id)| ClientShellPane {
            pane_id: (*id).into(),
            workspace_id: "w1".into(),
            tab_id: "t1".into(),
            label: None,
            cwd: None,
            foreground_cwd: None,
            focused: at == 0,
            right_click_passthrough: false,
        }));
    snapshot
}

#[test]
fn tab_target_retains_membership_and_rejects_stale_snapshots() {
    let original = snapshot();
    let target = Target::capture(&original, "t1").unwrap();
    assert_eq!(target.panes, ["p1".to_owned(), "p2".to_owned()]);
    let mut snapshot = original.clone();
    snapshot.focused_tab_id = None;
    snapshot.focused_workspace_id = None;
    assert!(target.validate(&snapshot).is_ok());
    assert_eq!(
        target.rename_params("  \u{4e2d}  ").unwrap(),
        json!({"tab_id": "t1", "label": "\u{4e2d}"})
    );
    assert!(matches!(
        target.rename_params(" \u{2003}\t"),
        Err(Error::EmptyTabName)
    ));
    snapshot.boot_id.push_str("-replaced");
    assert!(matches!(target.validate(&snapshot), Err(Error::StaleTab)));
    snapshot = original.clone();
    snapshot.tabs[1].workspace_id.push_str("-moved");
    assert!(matches!(target.validate(&snapshot), Err(Error::StaleTab)));
    snapshot = original.clone();
    snapshot.workspaces.clear();
    assert!(matches!(target.validate(&snapshot), Err(Error::StaleTab)));
    snapshot = original;
    snapshot.tabs.clear();
    assert!(matches!(target.validate(&snapshot), Err(Error::StaleTab)));
    assert!(Target::capture(&snapshot, "missing").is_none());
}

#[test]
fn a_move_needs_the_panes_the_tab_held() {
    let target = Target::capture(&snapshot(), "t1").unwrap();
    assert!(target.validate_move(&snapshot()).is_ok());
    for panes in [
        &[][..],
        &["p1"][..],
        &["p1", "p3"][..],
        &["p1", "p2", "p3"][..],
    ] {
        assert!(
            matches!(
                target.validate_move(&snapshot_with_tab_panes(panes)),
                Err(Error::StaleTab)
            ),
            "{panes:?}"
        );
    }
    // The same panes in another order still move.
    assert!(
        target
            .validate_move(&snapshot_with_tab_panes(&["p2", "p1"]))
            .is_ok()
    );
}

#[test]
fn the_rows_put_this_clients_mute_before_closing() {
    let snapshot = snapshot();
    let target = Target::capture(&snapshot, "t1").unwrap();
    assert_eq!(
        target.actions(Offers {
            movable: false,
            mute: Action::Mute,
        }),
        [Action::NewTab, Action::Rename, Action::Mute, Action::Close]
    );
    assert_eq!(
        target.actions(Offers {
            movable: true,
            mute: Action::UnmuteWorkspace,
        }),
        [
            Action::NewTab,
            Action::Rename,
            Action::MoveToWorkspace,
            Action::UnmuteWorkspace,
            Action::Close,
        ]
    );
    // A tab with no panes has nothing to move.
    let paneless = Target::capture(&snapshot_with_tab_panes(&[]), "t1").unwrap();
    assert!(
        !paneless
            .actions(Offers {
                movable: true,
                mute: Action::Mute,
            })
            .contains(&Action::MoveToWorkspace)
    );
}
