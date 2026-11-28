// ABOUTME: Exercises late replies and call registration during connection loss.
// ABOUTME: Checks real framed CLOSE writes and the bounded pending-call fence.

use super::*;

#[test]
fn late_stream_answers_close_the_core_channel() {
    for method in [Method::TerminalAttach, Method::EventsSubscribe] {
        let (client, mut core) = connected();
        let client = Arc::new(client);
        let call = thread::spawn({
            let client = Arc::clone(&client);
            move || client.call_with_timeout(method, &json!({}), false, Duration::from_millis(20))
        });
        let request = core.request();
        assert!(matches!(call.join().unwrap(), Err(Error::Timeout(found)) if found == method));
        core.answer(&request, json!({"channel": 7}));
        let close = core.frame();
        assert_eq!(
            (close.channel, close.kind, close.payload),
            (7, Kind::Close, vec![])
        );
    }
}

pub(super) fn answer_worker() -> Worker<io::Cursor<Vec<u8>>> {
    let (outbound, queue) = crossbeam_channel::bounded(1);
    outbound.try_send(vec![]).unwrap();
    Worker {
        closing: VecDeque::new(),
        decoder: Decoder::default(),
        queue,
        shared: Arc::new(Shared::default()),
        stop: Arc::new(AtomicBool::new(false)),
        stream: io::Cursor::new(Vec::new()),
        written: 0,
    }
}

fn assert_unheard_stream_closed(full: bool) {
    let mut worker = answer_worker();
    let (waiter, receiver) = crossbeam_channel::bounded(1);
    if full {
        waiter
            .try_send(Reply {
                result: Ok(Value::Null),
                route: None,
            })
            .unwrap_or_else(|_| panic!("empty reply queue"));
    } else {
        drop(receiver);
    }
    locked(&worker.shared.pending).insert(
        1,
        Call::Sent(WaitingCall {
            method: Method::TerminalAttach,
            resize_notifications: false,
            waiter,
        }),
    );
    worker
        .answer(br#"{"id":1,"result":{"channel":7}}"#)
        .unwrap();
    let mut decoder = Decoder::default();
    decoder.push(worker.stream.get_ref());
    let close = decoder
        .next_frame()
        .unwrap()
        .expect("unclaimed channel must close");
    assert_eq!((close.channel, close.kind), (7, Kind::Close));
    assert!(locked(&worker.shared.routes).is_empty());
}

#[test]
fn a_disconnected_waiter_closes_its_stream_with_a_full_outbound_queue() {
    assert_unheard_stream_closed(false);
}

#[test]
fn a_full_waiter_closes_its_stream_with_a_full_outbound_queue() {
    assert_unheard_stream_closed(true);
}

#[test]
fn loss_before_registration_is_lost_instead_of_a_timeout() {
    let (client, _core) = connected();
    let client = Arc::new(client);
    let mut pending = locked(&client.shared.pending);
    let call = thread::spawn({
        let client = Arc::clone(&client);
        move || client.list(false)
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    while client.next_id.load(Ordering::Relaxed) == 1 {
        assert!(Instant::now() < deadline, "call never reached registration");
        thread::yield_now();
    }
    client.shared.lost.store(true, Ordering::Release);
    pending.clear();
    drop(pending);
    assert!(matches!(call.join().unwrap(), Err(Error::Lost)));
}

#[test]
fn malformed_stream_metadata_closes_the_unclaimed_channel() {
    for method in [Method::TerminalAttach, Method::EventsSubscribe] {
        let (client, mut core) = connected();
        let client = Arc::new(client);
        let call = thread::spawn({
            let client = Arc::clone(&client);
            move || match method {
                Method::TerminalAttach => client
                    .attach(&wire::Attach {
                        terminal: "t7",
                        ..wire::Attach::default()
                    })
                    .map(drop),
                _ => client.subscribe().map(drop),
            }
        });
        let request = core.request();
        core.answer(&request, json!({"channel": 7}));
        assert!(matches!(call.join().unwrap(), Err(Error::Invalid(found, _)) if found == method));
        let close = core.frame();
        assert_eq!((close.channel, close.kind), (7, Kind::Close));
        core.send(7, Kind::Close, b"");
        let next = thread::spawn({
            let client = Arc::clone(&client);
            move || client.doc_get("after-invalid-stream", None)
        });
        let frame = core.frame();
        assert_eq!((frame.channel, frame.kind), (0, Kind::Data));
        let request = serde_json::from_slice(&frame.payload).unwrap();
        core.answer(&request, json!({"value": null, "version": 0}));
        assert!(next.join().unwrap().is_ok());
        assert!(locked(&client.shared.routes).is_empty());
    }
}
