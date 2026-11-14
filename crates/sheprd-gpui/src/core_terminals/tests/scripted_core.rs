// ABOUTME: A scripted Core on a private socket, for tests that need a real backend.
// ABOUTME: Answers only the handshake, naming the Core ID the test chose.

use sheprd_core_client::{Core, CoreBackend, Endpoint, Method};
use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    time::{Duration, Instant},
};

/// A backend whose handshake named `core_id`, and the scripted Core's end.
pub(crate) fn connect(core_id: &str) -> (CoreBackend, UnixStream) {
    let parent = std::env::var_os("HERDR_TEST_TMPDIR").unwrap_or_else(|| "/tmp".into());
    let dir = tempfile::Builder::new()
        .prefix("hcid")
        .tempdir_in(parent)
        .unwrap();
    let socket = dir.path().join("c.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let methods: Vec<&str> = Method::ALL.iter().map(|method| method.as_str()).collect();
    let reply = serde_json::json!({"jsonrpc": "2.0", "id": 0, "result": {
        "generation": 1, "version": "0.1.13", "framed": true, "features": [],
        "core": {"id": core_id, "name": "box"}, "methods": methods}});
    let peer = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut byte = [0];
        while byte[0] != b'\n' {
            stream.read_exact(&mut byte).unwrap();
        }
        let mut answer = serde_json::to_vec(&reply).unwrap();
        answer.push(b'\n');
        stream.write_all(&answer).unwrap();
        stream
    });
    let core = Core::connect(&Endpoint {
        socket,
        session: "identity".into(),
    })
    .unwrap();
    (CoreBackend::new(core, 100), peer.join().unwrap())
}

/// Drops the scripted Core's end and waits until `backend` reads as lost.
pub(crate) fn lose(backend: &CoreBackend, peer: UnixStream) {
    drop(peer);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !backend.core().is_lost() {
        assert!(Instant::now() < deadline, "the connection never dropped");
        std::thread::yield_now();
    }
}
