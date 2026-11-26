// ABOUTME: Checks when a Herdr roster observation reaches the mobile view.
// ABOUTME: Verifies freshness through local actions, paired surfaces and resume.
use super::*;
use std::time::Instant;

#[test]
fn roster_time_tracks_the_displayed_snapshot_through_buffering_and_local_navigation() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    assert_eq!(state.view().roster_observed_at_ms, None);
    let mut snap = snapshot();
    state.snapshot_at(Arc::new(snap.clone()), 1_000);
    state.surface(Arc::new(surface()));
    assert_eq!(state.view().roster_observed_at_ms, Some(1_000));
    snap.revision += 1;
    state.snapshot_at(Arc::new(snap.clone()), 2_000);
    assert_eq!(state.view().roster_observed_at_ms, Some(1_000));
    let mut frame = surface();
    frame.projection_revision = snap.revision;
    state.surface(Arc::new(frame));
    assert_eq!(state.view().roster_observed_at_ms, Some(2_000));
    state.start_fence(
        "choose".into(),
        OwnedNavigationTarget::Pane("other".into()),
        Ok("barrier".into()),
    );
    assert_eq!(state.view().roster_observed_at_ms, Some(2_000));
    state.snapshot_at(Arc::new(snapshot()), 3_000);
    assert_eq!(state.view().roster_observed_at_ms, Some(2_000));
}

#[test]
fn a_verified_resume_refreshes_the_roster_after_the_correlated_daemon_reply() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot_at(Arc::new(snapshot()), 1_000);
    state.surface(Arc::new(surface()));
    state.park();
    let now = Instant::now();
    state.start_resume("current".into(), now);
    state.surface(Arc::new(surface()));
    assert!(!state.finish_resume("current", now).unwrap());
    assert_eq!(state.view().roster_observed_at_ms, Some(1_000));
    state.receipt(
        "current",
        &serde_json::json!({"result":{
            "type":"client_shell_surface_set", "active":true, "projection_revision":7
        }}),
    );
    assert!(state.finish_resume("current", now).unwrap());
    assert!(state.view().roster_observed_at_ms.unwrap() > 1_000);
}
