use super::*;
use crate::tests::{Endpoint, event};

#[test]
fn delayed_validation_and_zero_delay_attention_match_tui() {
    for (kind, status, delay, count) in [
        (Kind::NeedsAttention, AgentStatus::Blocked, 1, 1),
        (Kind::NeedsAttention, AgentStatus::Working, 1, 0),
        (Kind::NeedsAttention, AgentStatus::Working, 0, 1),
        (Kind::Finished, AgentStatus::Done, 1, 1),
        (Kind::Finished, AgentStatus::Idle, 0, 0),
        (Kind::Finished, AgentStatus::Working, 1, 0),
    ] {
        let now = Instant::now();
        let mut endpoint = Endpoint::new(status);
        endpoint.events.push_back((now, event(kind)));
        let mut policy = Policy::default();
        let mut sounds = Vec::new();
        policy.poll(endpoint.live(), Some(delay), false, now, |e| sounds.push(e));
        if delay > 0 {
            assert!(sounds.is_empty());
        }
        policy.poll(
            endpoint.live(),
            Some(delay),
            false,
            now + Duration::from_secs(1),
            |e| sounds.push(e),
        );
        assert_eq!(sounds.len(), count, "{kind:?}, {status:?}, {delay}");
        policy.poll(
            endpoint.live(),
            Some(delay),
            false,
            now + Duration::from_secs(5),
            |_| panic!("replayed sound"),
        );
    }
}

#[test]
fn completion_grace_rechecks_working_and_missing_agents_but_not_idle() {
    for missing in [false, true] {
        let now = Instant::now();
        let mut endpoint = Endpoint::new(AgentStatus::Working);
        if missing {
            Arc::make_mut(endpoint.snapshot.as_mut().unwrap())
                .agents
                .clear();
        }
        endpoint.events.push_back((now, event(Kind::Finished)));
        let mut policy = Policy::default();
        policy.poll(endpoint.live(), Some(0), false, now, |_| {
            panic!("premature completion")
        });
        assert_eq!(policy.pending.len(), 1);
        endpoint.snapshot = Endpoint::new(AgentStatus::Done).snapshot;
        let mut count = 0;
        policy.poll(
            endpoint.live(),
            Some(0),
            false,
            now + Duration::from_millis(50),
            |_| count += 1,
        );
        assert_eq!(count, 1);
    }
    let now = Instant::now();
    let mut endpoint = Endpoint::new(AgentStatus::Working);
    endpoint.events.push_back((now, event(Kind::Finished)));
    let mut policy = Policy::default();
    policy.poll(endpoint.live(), Some(0), false, now, |_| panic!());
    policy.poll(
        endpoint.live(),
        Some(0),
        false,
        now + Duration::from_secs(1),
        |_| panic!(),
    );
    assert!(policy.pending.is_empty());
    endpoint.snapshot = Endpoint::new(AgentStatus::Done).snapshot;
    policy.poll(
        endpoint.live(),
        Some(0),
        false,
        now + Duration::from_secs(2),
        |_| panic!("expired completion"),
    );
}

#[test]
fn focus_suppresses_only_finished_for_selected_focused_tab() {
    for kind in [
        Kind::Finished,
        Kind::NeedsAttention,
        Kind::Custom,
        Kind::UpdateInstalled,
    ] {
        for focused in [false, true] {
            for active_tab in [false, true] {
                let now = Instant::now();
                let mut endpoint = Endpoint::new(AgentStatus::Done);
                let mut notification = event(kind);
                if !active_tab {
                    notification.tab_id = Some("other-tab".into());
                }
                endpoint.events.push_back((now, notification));
                let mut count = 0;
                Policy::default().poll(endpoint.live(), Some(0), focused, now, |_| count += 1);
                assert_eq!(
                    count,
                    usize::from(!(kind == Kind::Finished && focused && active_tab))
                );
            }
        }
    }
}

#[test]
fn custom_is_immediate_and_each_immediate_event_moves_once() {
    let now = Instant::now();
    let mut endpoint = Endpoint::new(AgentStatus::Idle);
    endpoint.events.push_back((now, event(Kind::Custom)));
    endpoint.events.push_back((now, event(Kind::Custom)));
    let mut count = 0;
    let mut policy = Policy::default();
    policy.poll(endpoint.live(), Some(3600), true, now, |_| count += 1);
    assert_eq!(count, 2);
    assert!(endpoint.events.is_empty() && policy.pending.is_empty());
}

#[test]
fn replacement_is_per_endpoint_and_pane_even_without_sound() {
    let now = Instant::now();
    let mut first = Endpoint::new(AgentStatus::Blocked);
    let mut second = Endpoint::new(AgentStatus::Blocked);
    let mut policies = [Policy::default(), Policy::default()];
    for (policy, endpoint) in policies.iter_mut().zip([&mut first, &mut second]) {
        endpoint
            .events
            .push_back((now, event(Kind::NeedsAttention)));
        policy.poll(endpoint.live(), Some(1), false, now, |_| panic!());
    }
    let mut replacement = event(Kind::Custom);
    replacement.sound = None;
    first.events.push_back((now, replacement));
    policies[0].poll(
        first.live(),
        Some(1),
        false,
        now + Duration::from_secs(1),
        |_| panic!("replaced"),
    );
    let mut count = 0;
    policies[1].poll(
        second.live(),
        Some(1),
        false,
        now + Duration::from_secs(1),
        |_| count += 1,
    );
    assert_eq!(count, 1);
}

#[test]
fn no_pane_completion_is_rejected_and_workspace_focus_is_fallback() {
    let now = Instant::now();
    let mut endpoint = Endpoint::new(AgentStatus::Done);
    let mut notification = event(Kind::Finished);
    notification.pane_id = None;
    endpoint.events.push_back((now, notification));
    let mut policy = Policy::default();
    policy.poll(endpoint.live(), Some(0), false, now, |_| panic!());
    let mut notification = event(Kind::Finished);
    notification.tab_id = None;
    endpoint.events.push_back((now, notification));
    policy.poll(endpoint.live(), Some(0), true, now, |_| panic!());
}

#[test]
fn startup_buffer_and_pending_are_bounded() {
    let now = Instant::now();
    let mut endpoint = Endpoint::new(AgentStatus::Blocked);
    let mut policy = Policy::default();
    for _ in 0..3 {
        for i in 0..MAX_PENDING {
            let mut notification = event(Kind::NeedsAttention);
            notification.pane_id = Some(i.to_string());
            endpoint.events.push_back((now, notification));
        }
        policy.poll(endpoint.live(), None, false, now, |_| panic!());
    }
    assert_eq!(policy.incoming.len(), MAX_PENDING);
    policy.poll(endpoint.live(), Some(3600), false, now, |_| panic!());
    assert_eq!(policy.pending.len(), MAX_PENDING);
    assert!(policy.incoming.is_empty());
}
