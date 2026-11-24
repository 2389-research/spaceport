use super::*;
use crate::{
    Error,
    pane::{Action, Offers, Target},
};
use spaceport_client::Method;

#[test]
fn captured_pane_payloads_ignore_focus_and_reject_stale_membership() {
    let original = snapshot();
    let target = Target::capture(&original, "p1").unwrap();
    let mut snapshot = original.clone();
    snapshot.focused_pane_id = None;
    snapshot.focused_tab_id = None;
    snapshot.focused_workspace_id = None;
    assert!(target.validate(&snapshot).is_ok());
    assert_eq!(
        target.rename_params("  \u{4e2d}  "),
        json!({"pane_id": "p1", "label": "\u{4e2d}"})
    );
    assert_eq!(
        target.rename_params(" \u{2003}\t"),
        json!({"pane_id": "p1", "label": ""})
    );
    for (action, direction) in [(Action::SplitRight, "right"), (Action::SplitDown, "down")] {
        assert_eq!(
            action.request(&target).unwrap(),
            (
                Method::PaneSplit,
                json!({"target_pane_id": "p1", "direction": direction, "focus": true})
            )
        );
    }
    assert_eq!(
        Action::Zoom.request(&target).unwrap(),
        (Method::PaneZoom, json!({"pane_id": "p1", "mode": "on"}))
    );
    let mut zoomed = original.clone();
    zoomed.tabs[1].zoomed = true;
    let target_zoomed = Target::capture(&zoomed, "p1").unwrap();
    assert_eq!(
        Action::Zoom.request(&target_zoomed).unwrap(),
        (Method::PaneZoom, json!({"pane_id": "p1", "mode": "off"}))
    );
    // The focused pane takes the menu's pane's place.
    assert_eq!(
        Action::Swap.request(&target).unwrap(),
        (
            Method::PaneSwap,
            json!({"source_pane_id": "p2", "target_pane_id": "p1"})
        )
    );
    let focused = Target::capture(&original, "p2").unwrap();
    assert!(focused.focused.is_none());
    assert!(Action::Swap.request(&focused).is_none());
    assert_eq!(
        Action::RightClick.request(&target).unwrap(),
        (
            Method::PaneInputSet,
            json!({"pane_id": "p1", "right_click": "pane"})
        )
    );
    let mut routed = original.clone();
    routed.panes[1].right_click_passthrough = true;
    let routed = Target::capture(&routed, "p1").unwrap();
    assert_eq!(
        Action::RightClick.request(&routed).unwrap(),
        (
            Method::PaneInputSet,
            json!({"pane_id": "p1", "right_click": "herdr"})
        )
    );
}

#[test]
fn a_pane_target_rejects_a_snapshot_that_moved_under_it() {
    let original = snapshot();
    let target = Target::capture(&original, "p1").unwrap();
    for case in 0..7 {
        let mut snapshot = original.clone();
        match case {
            0 => snapshot.boot_id.push('x'),
            1 => snapshot.workspaces.clear(),
            2 => snapshot.tabs.clear(),
            3 => snapshot.tabs[1].workspace_id.push('x'),
            4 => snapshot.panes[1].tab_id.push('x'),
            5 => snapshot.panes[1].workspace_id.push('x'),
            _ => snapshot.panes.truncate(1),
        }
        assert!(matches!(target.validate(&snapshot), Err(Error::StalePane)));
    }
    assert!(Target::capture(&original, "missing").is_none());
}

#[test]
fn the_rows_need_another_pane_a_host_and_a_picker() {
    let snapshot = snapshot();
    let target = Target::capture(&snapshot, "p1").unwrap();
    assert_eq!(
        target.actions(Offers::default()),
        [
            Action::Rename,
            Action::SplitRight,
            Action::SplitDown,
            Action::Swap,
            Action::Zoom,
            Action::EditScrollback,
            Action::RightClick,
            Action::Close,
        ]
    );
    assert_eq!(
        target.actions(Offers {
            movable: true,
            destinations: false,
            processes: false,
        }),
        [
            Action::Rename,
            Action::SplitRight,
            Action::SplitDown,
            Action::Swap,
            Action::Zoom,
            Action::MoveToNewTab,
            Action::MoveToNewWorkspace,
            Action::EditScrollback,
            Action::RightClick,
            Action::Close,
        ]
    );
    // The picker is offered only when it has somewhere to list.
    assert_eq!(
        target
            .actions(Offers {
                movable: true,
                destinations: true,
                processes: true,
            })
            .len(),
        12
    );
    // A row this machine cannot act on is not offered.
    let focused = Target::capture(&snapshot, "p2").unwrap();
    let rows = focused.actions(Offers::default());
    assert!(!rows.contains(&Action::Swap));
    assert!(rows.contains(&Action::SplitRight));
}
