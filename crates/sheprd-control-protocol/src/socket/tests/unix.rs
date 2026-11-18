// ABOUTME: Checks private Unix socket permissions and native OS path handling.
// ABOUTME: Exercises invalid filename bytes on Linux, where the filesystem permits them.
use super::*;
use std::os::unix::{fs::PermissionsExt as _, net::UnixDatagram};

#[test]
fn only_the_user_can_reach_the_socket() {
    let address = address();
    let _server = Server::bind(&address.path).unwrap();
    let mode = std::fs::metadata(&address.path)
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[test]
fn a_stale_socket_is_replaced() {
    let address = address();
    // A socket file nobody listens on, as a crashed app leaves behind. A
    // datagram socket leaves the same stale file and never answers a stream
    // connection, so no other test's forked child can hold the address open
    // while this one binds it.
    drop(UnixDatagram::bind(&address.path).unwrap());
    Server::bind(&address.path).unwrap();
}

#[test]
fn socket_address_keeps_the_exact_os_path_bytes() {
    use std::os::unix::ffi::OsStrExt as _;
    let state = Path::new(std::ffi::OsStr::from_bytes(b"/tmp/control-\xff"));
    assert_eq!(
        crate::socket::address(state).unwrap(),
        state.join("control.sock")
    );
}

// macOS rejects these filename bytes with EILSEQ before a socket can bind.
#[cfg(target_os = "linux")]
#[test]
fn non_utf8_socket_paths_keep_their_exact_os_bytes() {
    use std::os::unix::ffi::OsStrExt as _;
    let address = address();
    let path = address
        .path
        .parent()
        .unwrap()
        .join(std::ffi::OsStr::from_bytes(b"control-\xff.sock"));
    let server = Server::bind(&path).unwrap();
    let caller = thread::spawn(move || call(&path, &focus_request()));
    server.next().unwrap().respond(Response::done("focused"));
    assert_eq!(caller.join().unwrap().unwrap(), Response::done("focused"));
}
