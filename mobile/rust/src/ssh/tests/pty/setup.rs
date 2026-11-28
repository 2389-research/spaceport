// ABOUTME: Tests SSH channel setup with the pane lookup held open by the host.
// ABOUTME: Covers early input, misdirected lookup replies, and limited session slots.
use super::*;

const BOUND: Duration = Duration::from_secs(2);

fn bridge(server: &peer::Server) -> (UnixStream, transport::BridgeGuard, Arc<PtyRoutes>) {
    let routes = Arc::new(PtyRoutes::default());
    routes.accept_path("/fixture/herdr".into());
    let (stream, guard) = transport::open_bridge(
        server.host(),
        server.password(),
        "cat".into(),
        &AtomicBool::new(false),
        None,
        Some((routes.clone(), "studio".into())),
    )
    .unwrap();
    (stream, guard, routes)
}

fn attach(routes: &PtyRoutes) -> Arc<MobilePty> {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(crate::tests::snapshot()));
    state.surface(Arc::new(crate::tests::surface()));
    let target = state.target().unwrap();
    routes
        .attach(Arc::new(Mutex::new(state)), target, 80, 24)
        .unwrap()
}

fn finish_lookup(peer: russh::server::Handle, channel: russh::ChannelId, pane: &str) {
    let reply = serde_json::json!({"result":{"pane":{"pane_id":pane,"terminal_id":"term_123"}}});
    runtime().unwrap().block_on(async {
        peer.data(channel, reply.to_string().into_bytes())
            .await
            .unwrap();
        peer.exit_status_request(channel, 0).await.unwrap();
        peer.eof(channel).await.unwrap();
        peer.close(channel).await.unwrap();
    });
}

fn first_output(pty: Arc<MobilePty>) -> Result<Vec<u8>, MobileError> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        loop {
            match pty.read_waiting() {
                Ok(bytes) if bytes.is_empty() => continue,
                result => {
                    let _ = tx.send(result);
                    break;
                }
            }
        }
    });
    rx.recv_timeout(BOUND).unwrap()
}

#[test]
fn prepares_the_pty_before_the_lookup_finishes_and_rejects_a_wrong_pane() {
    let server = peer::Server::start();
    let (_bridge, _guard, routes) = bridge(&server);
    let lookup = server.defer_next_lookup();
    let pty = attach(&routes);
    let (query, peer) = lookup.recv_timeout(BOUND).unwrap();
    let peer::ChannelEvent::AllocatedPty(prepared) = server
        .channel_events
        .recv_timeout(BOUND)
        .expect("allocate a PTY while the lookup is pending")
    else {
        panic!("the host should have received a PTY request");
    };

    assert_ne!(query, prepared);
    assert!(matches!(
        pty.write(b"early".to_vec()),
        Err(MobileError::TerminalStarting)
    ));
    finish_lookup(peer, query, "wrong");
    assert!(matches!(
        first_output(pty),
        Err(MobileError::TerminalLookup)
    ));
    loop {
        match server.channel_events.recv_timeout(BOUND).unwrap() {
            peer::ChannelEvent::Closed(id) if id == prepared => break,
            peer::ChannelEvent::Closed(_) => (),
            event => panic!("expected the unused PTY to close, got {event:?}"),
        }
    }
}

#[test]
fn a_host_with_one_free_session_can_attach_after_lookup_closes() {
    let server = peer::Server::start();
    let (_bridge, _guard, routes) = bridge(&server);
    let lookup = server.defer_next_lookup();
    let query_open = server.defer_next_open();
    let pty = attach(&routes);
    let (_query, grant) = query_open.recv_timeout(BOUND).unwrap();
    let overlapping_open = server.defer_next_open();
    runtime().unwrap().block_on(grant.accept());
    let (query, peer) = lookup.recv_timeout(BOUND).unwrap();
    let (_channel, denial) = overlapping_open.recv_timeout(BOUND).unwrap();
    runtime()
        .unwrap()
        .block_on(denial.reject(russh::ChannelOpenFailure::ResourceShortage));
    finish_lookup(peer, query, "w1:p1");
    assert_eq!(first_output(pty).unwrap(), b"attached");
    assert_eq!(server.attempts.load(Ordering::Acquire), 1);
}
