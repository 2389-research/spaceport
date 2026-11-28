// ABOUTME: Exercises cancellation and host pinning at the roster lookup entry point.
// ABOUTME: Checks the loopback SSH peer receives no authentication for either refusal.

use super::*;
use crate::{MobileError, MobileRosterRequest};

#[test]
fn cancelled_and_unpinned_observations_send_no_credentials() {
    let server = peer::Server::start();
    let request = MobileRosterRequest::new();
    request.cancel();
    assert!(matches!(
        request.read(server.host(), server.password(), "default".into()),
        Err(MobileError::Disconnected)
    ));
    let mut host = server.host();
    host.fingerprint = None;
    assert!(matches!(
        MobileRosterRequest::new().read(host, server.password(), "default".into()),
        Err(MobileError::Disconnected)
    ));
    assert_eq!(server.attempts.load(Ordering::Acquire), 0);
}

#[test]
fn cancellation_interrupts_ssh_while_the_channel_open_is_pending() {
    let server = peer::Server::start();
    let gate = server.defer_next_open();
    let host = server.host();
    let auth = server.password();
    let request = MobileRosterRequest::new();
    let read = request.clone();
    let (finished, result) = std::sync::mpsc::sync_channel(1);
    let thread = std::thread::spawn(move || {
        finished
            .send(read.read(host, auth, "default".into()).is_err())
            .unwrap();
    });
    let pending = gate.recv_timeout(Duration::from_secs(5)).unwrap();
    request.cancel();
    assert!(result.recv_timeout(Duration::from_secs(5)).unwrap());
    drop(pending);
    thread.join().unwrap();
}
