// ABOUTME: Pins when a chat pane connects again after its connection fails or is refused.
// ABOUTME: Covers exact retry times, errors that end retries, and the connection counter.
use super::*;

/// Reconciles `p1` as `target` at `now` with a connector that refuses with
/// `error`, returning how many connections it was asked for.
fn refused(chats: &mut Chats, target: Target, now: Instant, error: impl Fn() -> Error) -> usize {
    let mut attempts = 0;
    chats.reconcile(vec![target], now, &mut |_, _, _| {
        attempts += 1;
        Err(error())
    });
    attempts
}

fn generation(chats: &Chats) -> u64 {
    chats.pane("p1").unwrap().generation()
}

fn observed() -> Target {
    let mut target = target("p1", A);
    target.existing = Some(existing::Binding {
        pane_id: "p1".into(),
        agent: sheprd_chat_protocol::Agent::Codex,
        session_id: "01234567-89ab-4cde-8123-456789abcdef".into(),
        herdr_session: None,
        pid: 21,
        shell_pid: 20,
        started_ticks: 123,
    });
    target
}

#[test]
fn a_lost_connection_is_retried_once_at_its_retry_time() {
    let mut harness = Harness::new();
    let stream = harness.attach(cursor(1, 0), vec![agent(1, "a")]);
    drop(stream);
    harness.poll_until(|chats| !chats.pane("p1").unwrap().attached());
    let lost = harness.now;
    harness.now = lost + Duration::from_millis(499);
    harness.reconcile(vec![target("p1", A)]);
    assert_eq!(harness.sinces.len(), 1, "not before the retry time");
    // The transcript stays up while the pane waits.
    assert!(harness.chats.pane("p1").unwrap().available());
    assert_eq!(text(&harness.chats), ["a"]);
    harness.now = lost + Duration::from_millis(500);
    harness.reconcile(vec![target("p1", A)]);
    assert_eq!(harness.sinces.len(), 2, "at the retry time");
    assert!(harness.chats.pane("p1").unwrap().has_link());
    harness.later();
    harness.reconcile(vec![target("p1", A)]);
    assert_eq!(harness.sinces.len(), 2, "a connected pane connects no more");
}

#[test]
fn each_refused_connection_waits_longer_and_counts_once() {
    let mut chats = Chats::default();
    let start = Instant::now();
    let closed = || Error::Closed;
    assert_eq!(refused(&mut chats, target("p1", A), start, closed), 1);
    assert_eq!(generation(&chats), 1);
    assert!(!chats.pane("p1").unwrap().has_link());
    assert_eq!(refused(&mut chats, target("p1", A), start, closed), 0);
    let early = start + Duration::from_millis(499);
    assert_eq!(refused(&mut chats, target("p1", A), early, closed), 0);
    assert_eq!(generation(&chats), 1);
    let second = start + Duration::from_millis(500);
    assert_eq!(refused(&mut chats, target("p1", A), second, closed), 1);
    assert_eq!(generation(&chats), 2);
    let early = second + Duration::from_millis(999);
    assert_eq!(refused(&mut chats, target("p1", A), early, closed), 0);
    let third = second + Duration::from_secs(1);
    assert_eq!(refused(&mut chats, target("p1", A), third, closed), 1);
    assert_eq!(generation(&chats), 3);
}

#[test]
fn a_pane_past_the_failure_limit_keeps_trying_in_the_background() {
    let mut chats = Chats::default();
    let mut now = Instant::now();
    let closed = || Error::Closed;
    for _ in 0..MAX_FAILURES {
        assert_eq!(refused(&mut chats, target("p1", A), now, closed), 1);
        now += Duration::from_secs(60);
    }
    assert!(!chats.pane("p1").unwrap().available());
    // The limit hides the chat view; it does not stop the retries.
    assert_eq!(refused(&mut chats, target("p1", A), now, closed), 1);
    assert_eq!(generation(&chats), u64::from(MAX_FAILURES) + 1);
}

#[test]
fn a_missing_socket_directory_is_never_retried() {
    let mut chats = Chats::default();
    let start = Instant::now();
    let missing = || Error::NoEndpointDir;
    assert_eq!(refused(&mut chats, target("p1", A), start, missing), 1);
    assert_eq!(refused(&mut chats, target("p1", A), start, missing), 0);
    let later = start + Duration::from_secs(3600);
    assert_eq!(refused(&mut chats, target("p1", A), later, missing), 0);
    assert_eq!(generation(&chats), 1);
    assert!(!chats.pane("p1").unwrap().available());
}

#[test]
fn only_an_observed_agent_that_exits_is_never_retried() {
    let exiting = || Error::Bye(ByeReason::Exiting);
    let start = Instant::now();
    let later = start + Duration::from_secs(3600);

    let mut chats = Chats::default();
    assert_eq!(refused(&mut chats, observed(), start, exiting), 1);
    assert_eq!(refused(&mut chats, observed(), later, exiting), 0);

    let mut chats = Chats::default();
    assert_eq!(refused(&mut chats, target("p1", A), start, exiting), 1);
    assert_eq!(refused(&mut chats, target("p1", A), later, exiting), 1);
}

#[test]
fn a_stream_that_stopped_fitting_is_retried_on_the_next_tick() {
    let mut chats = Chats::default();
    let now = Instant::now();
    let gap = || Error::Stream {
        after: None,
        got: cursor(1, 2),
    };
    // One connection per reconcile: never a second within the same one.
    for attempt in 1..=u64::from(MAX_FAILURES) + 2 {
        assert_eq!(refused(&mut chats, target("p1", A), now, gap), 1);
        assert_eq!(generation(&chats), attempt);
    }
    // It did not count toward backing off: an ordinary failure next waits
    // only the first delay.
    assert_eq!(
        refused(&mut chats, target("p1", A), now, || Error::Closed),
        1
    );
    let early = now + Duration::from_millis(499);
    assert_eq!(refused(&mut chats, target("p1", A), early, gap), 0);
    let due = now + Duration::from_millis(500);
    assert_eq!(refused(&mut chats, target("p1", A), due, gap), 1);
}
