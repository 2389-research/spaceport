// ABOUTME: Exercises resume proof against correlated Herdr replies and coherent projections.
// ABOUTME: Checks input epochs across standby, timeouts and overlapping resumes.
use super::*;
use std::time::{Duration, Instant};

fn ready() -> Projection {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    state.surface(Arc::new(surface()));
    state
}

fn receipt(state: &mut Projection, id: &str, revision: u64) {
    state.receipt(
        id,
        &serde_json::json!({"result":{
            "type":"client_shell_surface_set", "active":true, "projection_revision":revision
        }}),
    );
}

#[test]
fn resumed_input_requires_a_fresh_reply_and_its_projection() {
    let mut state = ready();
    let input = state.target().unwrap();
    let now = Instant::now();
    state.park();
    assert!(state.target().is_none());
    state.start_resume("request".into(), now);
    receipt(&mut state, "earlier", 7);
    assert!(!state.finish_resume("request", now).unwrap());
    receipt(&mut state, "request", 8);
    assert!(!state.finish_resume("request", now).unwrap());
    assert!(state.target().is_none());
    let mut next = snapshot();
    next.revision = 8;
    state.snapshot(Arc::new(next));
    let mut cells = surface();
    cells.projection_revision = 8;
    state.surface(Arc::new(cells));
    assert!(state.finish_resume("request", now).unwrap());
    let resumed = state.target().unwrap();
    assert_eq!(resumed.connection, input.connection);
    assert_eq!(resumed.pane_id, input.pane_id);
    assert_ne!(resumed, input);
}

#[test]
fn stale_proof_cannot_activate_a_new_resume_and_silence_expires() {
    let mut state = ready();
    let now = Instant::now();
    state.park();
    state.start_resume("one".into(), now);
    state.park();
    state.start_resume("two".into(), now);
    receipt(&mut state, "one", 7);
    assert!(state.finish_resume("one", now).is_err());
    assert!(!state.finish_resume("two", now).unwrap());
    assert!(
        state
            .finish_resume("two", now + Duration::from_secs(5))
            .is_err()
    );
    assert!(state.target().is_none());
}

#[test]
fn rejected_resume_does_not_unlock_input() {
    let mut state = ready();
    let now = Instant::now();
    state.park();
    state.start_resume("current".into(), now);
    state.rejected(Some("current"), "peer rejected the barrier".into());
    assert!(state.finish_resume("current", now).is_err());
    assert!(state.target().is_none());
}

#[test]
fn fresh_resume_ack_does_not_cancel_an_unsettled_focus_fence() {
    let mut state = ready();
    let now = Instant::now();
    state.start_fence(
        "choose".into(),
        Navigation::Pane("not-here-yet".into()),
        "focus-barrier".into(),
    );
    state.park();
    state.start_resume("check".into(), now);
    receipt(&mut state, "check", 7);
    assert!(!state.finish_resume("check", now).unwrap());
    assert!(state.target().is_none());
}

#[test]
fn a_terminal_needs_a_new_frame_but_an_empty_tab_can_resume_without_one() {
    let mut state = ready();
    let now = Instant::now();
    state.reported_focus = true;
    state.park();
    state.surface(Arc::new(surface()));
    assert!(
        state.view().terminal.is_none(),
        "discard in-flight parked frames"
    );
    assert!(!state.reported_focus);
    state.start_resume("again".into(), now);
    receipt(&mut state, "again", 7);
    assert!(!state.finish_resume("again", now).unwrap());
    let mut empty = snapshot();
    empty.revision = 8;
    empty.focused_pane_id = None;
    empty.panes.clear();
    state.snapshot(Arc::new(empty));
    assert!(state.finish_resume("again", now).unwrap());
    assert!(state.target().is_none());
}

#[test]
fn a_peer_that_answered_is_not_a_missing_transport_even_if_its_view_stays_stale() {
    let mut state = ready();
    let now = Instant::now();
    state.park();
    state.start_resume("new".into(), now);
    receipt(&mut state, "new", 99);
    assert!(matches!(
        state.finish_resume("new", now + Duration::from_secs(5)),
        Err(MobileError::ResumeUnconfirmed)
    ));
}
