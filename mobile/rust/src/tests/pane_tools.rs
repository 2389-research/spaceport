// ABOUTME: Exercises the pane trail and guarded actions from mobile's projection.
// ABOUTME: Models a second client changing focus and moving a selected pane.
use super::*;
use crate::controller::pane_tools::{MobilePaneAction, MobilePaneActions};
use spaceport_client::{Method, pane_history::Step};

#[test]
fn history_tracks_remote_focus_and_holds_input_until_arrival() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    let mut snap = snapshot();
    let mut frame = surface();
    let mut other = snap.panes[0].clone();
    other.pane_id = "other".into();
    snap.panes.push(other);
    state.snapshot(Arc::new(snap.clone()));
    state.surface(Arc::new(frame.clone()));
    snap.focused_pane_id = Some("other".into());
    snap.revision += 1;
    frame.projection_revision = snap.revision;
    frame.panes[0].pane_id = "other".into();
    state.snapshot(Arc::new(snap.clone()));
    state.surface(Arc::new(frame));
    assert_eq!(state.view().back_pane.as_deref(), Some("w1:p1"));
    assert!(state.begin_travel("boot-v1", "other", Step::Back).is_err());
    state.begin_travel("boot-v1", "w1:p1", Step::Back).unwrap();
    assert!(state.target().is_none());
    snap.panes.retain(|p| p.pane_id != "w1:p1");
    snap.revision += 1;
    state.snapshot(Arc::new(snap));
    assert!(state.go_to_step().is_err());
}

#[test]
fn actions_require_advertisement_and_a_current_workspace_tab_and_pane() {
    let mut snap = snapshot();
    let pane = snap.panes[0].clone();
    let caps = MobilePaneActions::from_methods(&["pane.split".into(), "pane.zoom".into()]);
    let request = |snap: &ClientShellSnapshot, action: MobilePaneAction| {
        caps.request(
            snap,
            &snap.boot_id,
            &pane.workspace_id,
            &pane.tab_id,
            &pane.pane_id,
            action,
        )
    };
    assert_eq!(
        request(&snap, MobilePaneAction::SplitDown).unwrap(),
        (
            Method::PaneSplit,
            serde_json::json!({"target_pane_id": pane.pane_id, "direction":"down","focus":true})
        )
    );
    assert!(
        request(
            &snap,
            MobilePaneAction::Rename {
                label: "foreign".into()
            }
        )
        .is_err()
    );
    snap.panes[0].workspace_id = "moved".into();
    assert!(request(&snap, MobilePaneAction::Zoom { enabled: true }).is_err());
}

#[test]
fn a_replacement_connection_restores_only_the_verified_session_and_boot() {
    use crate::controller::pane_tools::TrailHost;
    fn state() -> Projection {
        Projection::new(geometry(3, 2, 8, 16).unwrap())
    }
    let host = SshHost {
        hostname: "host".into(),
        username: "pilot".into(),
        port: 22,
        fingerprint: Some("verified".into()),
    };
    let identity = TrailHost::new(&host, "A");
    let mut previous = state();
    previous.trail_host = identity.clone();
    let mut snap = snapshot();
    let mut frame = surface();
    let mut pane = snap.panes[0].clone();
    pane.pane_id = "second".into();
    snap.panes.push(pane);
    previous.snapshot(Arc::new(snap.clone()));
    previous.surface(Arc::new(frame.clone()));
    snap.revision += 1;
    snap.focused_pane_id = Some("second".into());
    frame.panes[0].pane_id = "second".into();
    frame.projection_revision = snap.revision;
    previous.snapshot(Arc::new(snap.clone()));
    previous.surface(Arc::new(frame.clone()));
    let saved = previous.save_pane_trail().unwrap();
    let mut next = state();
    next.trail_host = TrailHost::new(&host, "B");
    next.snapshot(Arc::new(snap.clone()));
    next.surface(Arc::new(frame.clone()));
    next.restore_pane_trail(&saved);
    assert!(next.view().back_pane.is_none());
    next.trail_host = identity;
    next.restore_pane_trail(&saved);
    assert_eq!(next.view().back_pane.as_deref(), Some("w1:p1"));
    snap.boot_id = "restarted".into();
    frame.boot_id = snap.boot_id.clone();
    next.snapshot(Arc::new(snap));
    next.surface(Arc::new(frame));
    next.restore_pane_trail(&saved);
    assert!(next.view().back_pane.is_none());
}
