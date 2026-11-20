// ABOUTME: Exercises stale socket recovery and exact bound-socket cleanup.
// ABOUTME: Uses private real Unix sockets and independent file-lock owners.
use super::*;
use std::{
    fs::{File, OpenOptions},
    os::unix::net::UnixListener,
};

#[test]
fn an_old_server_cannot_remove_a_replacement_socket() {
    let address = address();
    let first = Server::bind(&address.path).unwrap();
    let cleanup = first.cleanup();
    std::fs::remove_file(&address.path).unwrap();
    let _replacement = UnixListener::bind(&address.path).unwrap();
    cleanup.release();
    assert!(
        std::os::unix::net::UnixStream::connect(&address.path).is_ok(),
        "the old server removed a socket it did not bind"
    );
}

#[test]
fn stale_recovery_refuses_while_another_starter_owns_the_lock() {
    let address = address();
    drop(UnixListener::bind(&address.path).unwrap());
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(address.path.with_extension("lock"))
        .unwrap();
    lock.lock().unwrap();
    assert!(matches!(
        Server::bind(&address.path),
        Err(crate::Error::SocketInUse { .. })
    ));
    drop(lock);
}

#[test]
fn cleanup_releases_only_the_owned_socket_and_keeps_the_lock_file() {
    let address = address();
    let server = Server::bind(&address.path).unwrap();
    server.cleanup().release();
    assert!(!address.path.exists());
    let lock = File::open(address.path.with_extension("lock")).unwrap();
    assert!(matches!(
        lock.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
}

#[test]
fn concurrent_starters_keep_one_owner_of_a_stale_address() {
    let address = address();
    drop(UnixListener::bind(&address.path).unwrap());
    let ready = Arc::new(std::sync::Barrier::new(8));
    let starters: Vec<_> = (0..8)
        .map(|_| {
            let ready = ready.clone();
            let path = address.path.clone();
            thread::spawn(move || {
                ready.wait();
                transport::Listener::bind(&path)
            })
        })
        .collect();
    let results: Vec<_> = starters
        .into_iter()
        .map(|starter| starter.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .all(|error| matches!(error, crate::Error::SocketInUse { .. }))
    );
    assert!(std::os::unix::net::UnixStream::connect(&address.path).is_ok());
}
