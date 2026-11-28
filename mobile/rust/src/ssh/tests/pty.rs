// ABOUTME: Exercises direct terminal attachment through the existing SSH connection.
// ABOUTME: Checks target changes, byte flow, channel teardown and bounded input.
use super::*;
use crate::{InputTarget, MobileError, controller::geometry, projection::Projection};
use std::sync::Mutex;

#[test]
fn direct_attach_uses_one_authenticated_session_and_pins_each_write() {
    let server = peer::Server::start();
    let routes = Arc::new(PtyRoutes::default());
    routes.accept_path("/fixture/herdr".into());
    let (mut bridge, _guard) = transport::open_bridge(
        server.host(),
        server.password(),
        "cat".into(),
        &AtomicBool::new(false),
        None,
        Some((routes.clone(), "studio".into())),
    )
    .unwrap();
    bridge
        .set_read_timeout(Some(Duration::from_secs(4)))
        .unwrap();
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(crate::tests::snapshot()));
    state.surface(Arc::new(crate::tests::surface()));
    let target = state.target().unwrap();
    let state = Arc::new(Mutex::new(state));
    // Hold the first lookup's SSH channel confirmation while the operator selects again.
    let deferred = server.defer_next_open();
    let first = routes
        .attach(state.clone(), target.clone(), 80, 24)
        .unwrap();
    let (mut cancelled, delayed) = deferred.recv_timeout(Duration::from_secs(2)).unwrap();
    let pty = routes
        .attach(state.clone(), target.clone(), 80, 24)
        .unwrap();
    let wait = |expected: &[u8]| {
        let end = std::time::Instant::now() + Duration::from_secs(5);
        let mut received = Vec::new();
        while received.len() < expected.len() {
            received.extend(pty.read().unwrap());
            assert!(std::time::Instant::now() < end);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(received, expected);
    };
    wait(b"attached");
    runtime().unwrap().block_on(async {
        delayed.accept().await;
        let message = tokio::time::timeout(Duration::from_secs(2), cancelled.wait())
            .await
            .unwrap();
        assert!(matches!(message, None | Some(russh::ChannelMsg::Close)));
    });
    assert!(matches!(first.read(), Err(MobileError::TerminalEnded)));
    assert!(matches!(
        pty.write(vec![0; 65537]),
        Err(MobileError::TerminalQueueFull)
    ));
    pty.write(vec![0, 27, 255]).unwrap();
    wait(&[0, 27, 255]);
    pty.resize(83, 25).unwrap();
    wait(b"resized");
    assert_eq!(server.attempts.load(Ordering::Acquire), 1);
    let stale = InputTarget {
        input_epoch: target.input_epoch.wrapping_add(1),
        ..target
    };
    assert!(matches!(
        routes.attach(state.clone(), stale, 80, 24),
        Err(MobileError::StaleView)
    ));
    let original = state.lock().unwrap().clone();
    {
        let mut changed = state.lock().unwrap();
        changed.park();
        routes.invalidate(&changed);
    }
    assert!(matches!(pty.write(vec![b'x']), Err(MobileError::StaleView)));
    *state.lock().unwrap() = original;
    assert!(matches!(pty.read(), Err(MobileError::TerminalEnded)));
    pty.close();
    bridge.write_all(b"bridge still open").unwrap();
    let mut buf = [0; 17];
    bridge.read_exact(&mut buf).unwrap();
    assert_eq!(buf, *b"bridge still open");
}

mod setup;
