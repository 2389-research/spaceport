//! Checks the notice a connection to a host whose key is not checked raises.
//! Each handshake shows it again, and a checked host shows nothing.
use super::*;

fn welcome() -> ClientEvent {
    ClientEvent::Connected(
        spaceport_client::protocol::endpoint::EndpointServerWelcome {
            generation: 1,
            server_version: "test".into(),
            snapshot_codec: String::new(),
            surface_codec: String::new(),
            input_codec: String::new(),
            blob_codec: String::new(),
            methods: Vec::new(),
            capabilities: Vec::new(),
            error: None,
        },
    )
}

#[test]
fn an_unverified_host_is_named_once_its_handshake_is_accepted() {
    let mut state = LiveState::default();
    state.apply(ClientEvent::HostKeysUnverified, describe, localize);
    // The bridge reports before the handshake; nothing shows until it lands.
    assert!(state.notices.is_none());
    state.apply(welcome(), describe, localize);
    assert_eq!(
        state.notices.text(),
        Some(fl!("state-host-key-unverified").as_str())
    );
}

#[test]
fn a_connection_that_checked_its_host_key_says_nothing() {
    let mut state = LiveState::default();
    state.apply(welcome(), describe, localize);
    assert!(state.notices.is_none());
}

#[test]
fn every_connection_to_an_unverified_host_says_it_again() {
    let mut state = LiveState::default();
    state.apply(ClientEvent::HostKeysUnverified, describe, localize);
    state.apply(welcome(), describe, localize);
    assert!(state.notices.is_some());
    state.apply(
        ClientEvent::Disconnected {
            reason: "socket closed".into(),
            ssh: None,
        },
        describe,
        localize,
    );
    // The next connection reports its own bridge, and shows the notice again
    // rather than keeping the drop's reason.
    state.apply(ClientEvent::HostKeysUnverified, describe, localize);
    state.apply(welcome(), describe, localize);
    assert_eq!(
        state.notices.connection_reason(),
        None,
        "a new handshake keeps the drop's reason"
    );
    assert_eq!(
        state.notices.text(),
        Some(fl!("state-host-key-unverified").as_str())
    );
}
