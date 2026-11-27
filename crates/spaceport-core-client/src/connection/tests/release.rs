// ABOUTME: Checks that a dropped stream's CLOSE survives a full outbound queue.
// ABOUTME: Covers ordering, channel reuse, loss, and the last link closing.

use super::*;

/// A worker writing into memory, whose queue holds `capacity` frames.
fn worker(capacity: usize) -> (Worker<io::Cursor<Vec<u8>>>, Link) {
    let (outbound, queue) = crossbeam_channel::bounded(capacity);
    let worker = Worker {
        closing: Vec::new(),
        decoder: Decoder::default(),
        queue,
        shared: Arc::new(Shared::default()),
        stop: Arc::new(AtomicBool::new(false)),
        stream: io::Cursor::new(Vec::new()),
        written: 0,
    };
    (worker, Link { outbound })
}

fn register(worker: &Worker<io::Cursor<Vec<u8>>>, link: &Link, channel: u32) -> Stream {
    let route = Arc::new(Route::new(false));
    locked(&worker.shared.routes).insert(channel, Arc::clone(&route));
    Stream {
        channel,
        link: link.clone(),
        route,
        shared: Arc::clone(&worker.shared),
    }
}

/// Every frame the worker wrote, as (channel, kind, payload).
fn written(worker: &Worker<io::Cursor<Vec<u8>>>) -> Vec<(u32, Kind, Vec<u8>)> {
    let mut decoder = Decoder::default();
    decoder.push(worker.stream.get_ref());
    std::iter::from_fn(|| decoder.next_frame().unwrap())
        .map(|frame| (frame.channel, frame.kind, frame.payload))
        .collect()
}

fn attachment(stream: Stream) -> Box<dyn std::any::Any> {
    let terminal = serde_json::from_value(crate::tests::terminal("t5")).unwrap();
    Box::new(Attachment::new(stream, terminal, false))
}

fn subscription(stream: Stream) -> Box<dyn std::any::Any> {
    Box::new(Subscription::new(stream, 0, Vec::new()))
}

fn assert_released_behind_accepted_data(claim: fn(Stream) -> Box<dyn std::any::Any>) {
    let (mut worker, link) = worker(3);
    let released = claim(register(&worker, &link, 5));
    let other = attachment(register(&worker, &link, 6));
    link.send(5, Kind::Data, b"one").unwrap();
    link.send(5, Kind::Data, b"two").unwrap();
    link.send(6, Kind::Data, b"six").unwrap();
    assert!(matches!(
        link.send(6, Kind::Data, b"full"),
        Err(Error::Busy)
    ));

    drop(released);
    assert!(worker.write_queued().unwrap());
    let other = other.downcast::<Attachment>().unwrap();
    other.write(b"more").unwrap();
    assert!(worker.write_queued().unwrap());
    assert!(worker.write_queued().unwrap());

    // One CLOSE, after the data the queue had accepted, and no call on
    // channel 0: the terminal itself is never asked to end.
    assert_eq!(
        written(&worker),
        [
            (5, Kind::Data, b"one".to_vec()),
            (5, Kind::Data, b"two".to_vec()),
            (6, Kind::Data, b"six".to_vec()),
            (5, Kind::Close, vec![]),
            (6, Kind::Data, b"more".to_vec()),
        ]
    );
    let routes = locked(&worker.shared.routes);
    assert!(!routes.contains_key(&5));
    assert!(routes.contains_key(&6));
}

#[test]
fn a_dropped_attachment_closes_behind_a_full_queue() {
    assert_released_behind_accepted_data(attachment);
}

#[test]
fn a_dropped_subscription_closes_behind_a_full_queue() {
    assert_released_behind_accepted_data(subscription);
}

#[test]
fn a_release_never_closes_a_channel_the_core_reused() {
    let (mut worker, link) = worker(1);
    let released = attachment(register(&worker, &link, 5));
    drop(released);
    // The core closes 5 and hands it to a new stream before the worker runs.
    worker
        .dispatch(Frame {
            channel: 5,
            kind: Kind::Close,
            payload: Vec::new(),
        })
        .unwrap();
    let reused = register(&worker, &link, 5);
    assert!(worker.write_queued().unwrap());
    assert!(written(&worker).is_empty());
    assert!(Arc::ptr_eq(
        &locked(&worker.shared.routes)[&5],
        &reused.route
    ));
}

#[test]
fn the_last_stream_closes_before_the_worker_stops() {
    let (mut worker, link) = worker(1);
    let released = attachment(register(&worker, &link, 5));
    drop(link);
    drop(released);
    assert!(!worker.write_queued().unwrap());
    assert_eq!(written(&worker), [(5, Kind::Close, vec![])]);
}

#[test]
fn losing_the_connection_drops_pending_releases() {
    let (mut worker, link) = worker(1);
    drop(attachment(register(&worker, &link, 5)));
    let kept = attachment(register(&worker, &link, 6));
    worker.shared.lose();
    assert!(locked(&worker.shared.released).is_empty());
    drop(kept);
    assert!(locked(&worker.shared.released).is_empty());
    assert!(worker.write_queued().unwrap());
    assert!(written(&worker).is_empty());
}
