// ABOUTME: Checks idle timing boundaries, missing history and identity across snapshots.
// ABOUTME: Uses actual Herdr wire snapshots with a caller-controlled observation clock.
use super::*;
use spaceport_protocol::AgentStatus;

fn fixture() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(
        "../../../spaceport-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    ))
    .unwrap()
}

fn report(
    tracker: &mut IdleTracker,
    snapshot: &mut ClientShellSnapshot,
    status: AgentStatus,
    seq: u64,
    time: u64,
) {
    snapshot.revision += 1;
    snapshot.agents[0].agent_status = status;
    snapshot.agents[0].state_change_seq = seq;
    tracker.observe(snapshot, time);
}

#[test]
fn witnessed_transition_and_unchanged_done_to_idle_keep_the_start() {
    let mut tracker = IdleTracker::default();
    let mut snapshot = fixture();
    report(&mut tracker, &mut snapshot, AgentStatus::Working, 4, 1);
    report(&mut tracker, &mut snapshot, AgentStatus::Done, 5, 1200);
    report(&mut tracker, &mut snapshot, AgentStatus::Idle, 5, 9000);
    assert_eq!(
        tracker.get("w1:p1"),
        Some(IdleObservation {
            first_observed_ms: 1200,
            onset_observed: true
        })
    );
}

#[test]
fn discovery_and_changed_sequence_after_gap_only_bound_the_start() {
    let mut tracker = IdleTracker::default();
    let mut snapshot = fixture();
    report(&mut tracker, &mut snapshot, AgentStatus::Idle, 8, 15);
    tracker.interrupt();
    report(&mut tracker, &mut snapshot, AgentStatus::Done, 8, 200);
    assert_eq!(
        tracker.get("w1:p1"),
        Some(IdleObservation {
            first_observed_ms: 15,
            onset_observed: false
        })
    );
    report(&mut tracker, &mut snapshot, AgentStatus::Working, 9, 250);
    tracker.interrupt();
    report(&mut tracker, &mut snapshot, AgentStatus::Idle, 10, 300);
    assert_eq!(
        tracker.get("w1:p1"),
        Some(IdleObservation {
            first_observed_ms: 300,
            onset_observed: false
        })
    );
}

#[test]
fn reappearance_replacement_and_restart_never_inherit_idle_evidence() {
    let mut tracker = IdleTracker::default();
    let mut snapshot = fixture();
    report(&mut tracker, &mut snapshot, AgentStatus::Working, 4, 1);
    snapshot.boot_id = "next".into();
    report(&mut tracker, &mut snapshot, AgentStatus::Idle, 4, 2);
    assert_eq!(
        tracker.get("w1:p1"),
        Some(IdleObservation {
            first_observed_ms: 2,
            onset_observed: false
        })
    );
    snapshot.agents[0].agent = Some("replacement".into());
    report(&mut tracker, &mut snapshot, AgentStatus::Idle, 4, 3);
    assert_eq!(tracker.get("w1:p1").map(|o| o.first_observed_ms), Some(3));
    snapshot.agents.clear();
    snapshot.revision += 1;
    tracker.observe(&snapshot, 4);
    assert_eq!(tracker.get("w1:p1"), None);
}

#[test]
fn idle_groups_use_the_last_confirmation_and_exclude_unplaced_recent_agents() {
    let start = 2500;
    let hour = 3_600_000;
    for (onset_observed, young) in [
        (false, IdleAgeGroup::Unplaced),
        (true, IdleAgeGroup::Recent),
    ] {
        let observed = IdleObservation {
            first_observed_ms: start,
            onset_observed,
        };
        assert_eq!(observed.group(start - 1), young);
        assert_eq!(observed.group(start + hour - 1), young);
        assert_eq!(observed.group(start + hour), IdleAgeGroup::OneToFiveHours);
        assert_eq!(
            observed.group(start + 5 * hour - 1),
            IdleAgeGroup::OneToFiveHours
        );
        assert_eq!(
            observed.group(start + 5 * hour),
            IdleAgeGroup::FiveHoursOrMore
        );
    }
}
