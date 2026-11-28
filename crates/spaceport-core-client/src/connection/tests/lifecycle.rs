// ABOUTME: Checks bounded call states and atomic answer ownership.
// ABOUTME: Exercises cancellation, loss, and reply-versus-timeout ordering.

use super::*;

fn reply() -> Reply {
    Reply {
        result: Ok(json!({"value": 7})),
        route: None,
    }
}

#[test]
fn tombstones_keep_the_call_bound_until_the_late_answer() {
    let mut worker = abandoned::answer_worker();
    for id in 0..MAX_IN_FLIGHT as u64 {
        let answer = worker
            .shared
            .register(id, Method::EventsSubscribe, false)
            .unwrap();
        worker.shared.sent(id);
        assert!(matches!(
            worker
                .shared
                .wait(id, Method::EventsSubscribe, &answer, Duration::ZERO),
            Err(Error::Timeout(Method::EventsSubscribe))
        ));
    }
    assert!(matches!(
        worker
            .shared
            .register(MAX_IN_FLIGHT as u64, Method::TerminalList, false),
        Err(Error::Busy)
    ));
    worker
        .answer(br#"{"id":0,"result":{"channel":7}}"#)
        .unwrap();
    let answer = worker
        .shared
        .register(MAX_IN_FLIGHT as u64, Method::TerminalList, false)
        .unwrap();
    assert_eq!(locked(&worker.shared.pending).len(), MAX_IN_FLIGHT);
    drop(answer);
}

#[test]
fn an_answer_that_wins_the_timeout_fence_is_claimed() {
    let shared = Shared::default();
    let answer = shared.register(1, Method::DocGet, false).unwrap();
    shared.sent(1);
    assert!(matches!(
        shared.deliver(1, reply().result, None),
        Delivery::Answered
    ));
    let claimed = shared
        .finish(1, Method::DocGet, &answer, Err(RecvTimeoutError::Timeout))
        .unwrap();
    assert!(matches!(claimed.result, Ok(value) if value == json!({"value": 7})));
    assert!(locked(&shared.pending).is_empty());
}

#[test]
fn an_answer_before_the_sent_transition_remains_answered() {
    let shared = Shared::default();
    let answer = shared.register(1, Method::DocGet, false).unwrap();
    assert!(matches!(
        shared.deliver(1, reply().result, None),
        Delivery::Answered
    ));
    shared.sent(1);
    assert!(matches!(
        locked(&shared.pending).get(&1),
        Some(Call::Answered)
    ));
    let claimed = shared
        .wait(1, Method::DocGet, &answer, Duration::ZERO)
        .unwrap();
    assert!(matches!(claimed.result, Ok(value) if value == json!({"value": 7})));
    assert!(locked(&shared.pending).is_empty());
}

#[test]
fn loss_ends_waiters_and_tombstones_and_refuses_registration() {
    let shared = Shared::default();
    let abandoned = shared.register(1, Method::DocGet, false).unwrap();
    assert!(matches!(
        shared.wait(1, Method::DocGet, &abandoned, Duration::ZERO),
        Err(Error::Timeout(_))
    ));
    let waiting = shared.register(2, Method::DocGet, false).unwrap();
    let route = Arc::new(Route::new(false));
    locked(&shared.routes).insert(7, Arc::clone(&route));
    shared.lose();
    assert!(matches!(
        shared.wait(2, Method::DocGet, &waiting, Duration::ZERO),
        Err(Error::Lost)
    ));
    assert!(matches!(
        shared.register(3, Method::DocGet, false),
        Err(Error::Lost)
    ));
    assert!(locked(&shared.pending).is_empty());
    assert!(locked(&shared.routes).is_empty());
    let stream = Stream {
        channel: 7,
        link: Link {
            outbound: crossbeam_channel::bounded(1).0,
        },
        route,
        shared: Arc::new(shared),
    };
    // The stream has no public reader; its attachment exposes the real inbox.
    let terminal = serde_json::from_value(crate::tests::terminal("t1")).unwrap();
    let attachment = Attachment::new(stream, terminal, false);
    assert_eq!(
        attachment.read(Duration::ZERO).unwrap().end,
        Some(End::Lost)
    );
}

#[test]
fn an_unsent_call_does_not_leave_a_tombstone() {
    let shared = Shared::default();
    let answer = shared.register(1, Method::DocGet, false).unwrap();
    shared.cancel_unsent(1);
    assert!(locked(&shared.pending).is_empty());
    assert!(matches!(answer.try_recv(), Err(TryRecvError::Disconnected)));
}
