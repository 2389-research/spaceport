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
