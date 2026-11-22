// ABOUTME: Verifies that the mobile projection publishes local idle evidence through UniFFI.
// ABOUTME: Keeps a newly discovered idle agent distinct from a witnessed state change.
use super::*;

#[test]
fn mobile_rows_publish_idle_provenance_and_shared_age_bands() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    let mut snap = snapshot();
    snap.agents[0].agent_status = AgentStatus::Idle;
    state.snapshot(Arc::new(snap.clone()));
    let initial = state.view();
    let already_idle = &initial.panes[0];
    assert!(already_idle.idle_first_observed_ms.is_some());
    assert!(!already_idle.idle_onset_observed);

    snap.revision += 1;
    snap.agents[0].agent_status = AgentStatus::Working;
    state.snapshot(Arc::new(snap.clone()));
    assert!(state.view().panes[0].idle_first_observed_ms.is_none());
    snap.revision += 1;
    snap.agents[0].state_change_seq += 1;
    snap.agents[0].agent_status = AgentStatus::Done;
    state.snapshot(Arc::new(snap));
    let observed = state.view();
    let done = &observed.panes[0];
    assert!(done.idle_onset_observed);
    let start = done.idle_first_observed_ms.unwrap();
    assert_eq!(
        idle_age_group(Some(start), true, start + 3_600_000),
        IdleAgeGroup::OneToFiveHours
    );
    assert_eq!(idle_age_group(None, false, start), IdleAgeGroup::Unplaced);
}
