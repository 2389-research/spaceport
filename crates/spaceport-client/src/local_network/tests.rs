use super::*;

#[test]
fn only_a_refused_multicast_send_means_the_permission_is_missing() {
    assert_eq!(access(Ok(12)), LocalNetworkAccess::Allowed);
    // An app macOS can ask for sees no route; a process it cannot ask for,
    // such as a tool in a daemon's pane, sees a broken pipe.
    for kind in [io::ErrorKind::HostUnreachable, io::ErrorKind::BrokenPipe] {
        assert_eq!(access(Err(kind.into())), LocalNetworkAccess::Denied);
    }
    // No IPv4 network, or any other failure, says nothing either way.
    for kind in [
        io::ErrorKind::NetworkUnreachable,
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::AddrNotAvailable,
    ] {
        assert_eq!(access(Err(kind.into())), LocalNetworkAccess::Unknown);
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_platforms_have_no_local_network_permission() {
    assert_eq!(local_network_access(), LocalNetworkAccess::Allowed);
}
