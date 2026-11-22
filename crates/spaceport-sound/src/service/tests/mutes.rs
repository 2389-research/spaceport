use super::*;
use crate::tests::{Endpoint, event};
use spaceport_protocol::{AgentStatus, SemanticNotificationKind as Kind};

#[test]
fn a_muted_place_plays_nothing_and_unmuting_does_not_replay_it() {
    let (service, played) = service();
    let mut endpoint = Endpoint::new(AgentStatus::Idle);
    let mut policy = Policy::default();
    let now = Instant::now();
    endpoint.events.push_back((now, event(Kind::Custom)));
    service.poll(&mut policy, endpoint.live(), false, now, |_| true);
    assert!(played.try_recv().is_err());
    assert!(endpoint.events.is_empty());
    service.poll(&mut policy, endpoint.live(), false, now, |_| false);
    assert!(played.try_recv().is_err(), "nothing replays after unmuting");
    endpoint.events.push_back((now, event(Kind::Custom)));
    service.poll(&mut policy, endpoint.live(), false, now, |_| false);
    assert!(played.try_recv().is_ok());
}

#[test]
fn muting_while_a_sound_waits_out_its_delay_drops_it() {
    let (service, played) = service();
    let mut endpoint = Endpoint::new(AgentStatus::Blocked);
    let mut policy = Policy::default();
    let now = Instant::now();
    endpoint
        .events
        .push_back((now, event(Kind::NeedsAttention)));
    service.poll(&mut policy, endpoint.live(), false, now, |_| false);
    assert_eq!(policy.waiting(), 1, "waiting for the delay");
    let later = now + Duration::from_secs(2);
    service.poll(&mut policy, endpoint.live(), false, later, |_| true);
    assert_eq!(policy.waiting(), 0, "dropped while it waited");
    service.poll(&mut policy, endpoint.live(), false, later, |_| false);
    assert!(played.try_recv().is_err());
}

#[test]
fn only_the_muted_place_is_silent() {
    let (service, played) = service();
    let mut endpoint = Endpoint::new(AgentStatus::Idle);
    let mut policy = Policy::default();
    let now = Instant::now();
    let mut other = event(Kind::Custom);
    other.workspace_id = Some("other".into());
    other.tab_id = None;
    other.pane_id = None;
    endpoint.events.push_back((now, event(Kind::Custom)));
    endpoint.events.push_back((now, other));
    service.poll(&mut policy, endpoint.live(), false, now, |event| {
        event.workspace_id.as_deref() == Some("workspace")
    });
    let job = played.try_recv().unwrap();
    assert!(matches!(
        job.request,
        PlaybackRequest::Notification { event, .. } if event.workspace_id.as_deref() == Some("other")
    ));
    assert!(played.try_recv().is_err());
}
