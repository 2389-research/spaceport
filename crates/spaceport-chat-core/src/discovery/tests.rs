use super::*;
use spaceport_client::ConnectTarget;
use spaceport_live::{ConnectionStatus, LiveState};

fn connected(peer: bool) -> LiveState {
    LiveState {
        status: ConnectionStatus::Connected,
        local_daemon_peer: peer,
        ..LiveState::default()
    }
}

#[test]
fn only_an_ssh_daemon_is_reached_remotely() {
    let live = connected(true);
    let ssh = ConnectTarget::Ssh {
        target: "host".into(),
        session: "default".into(),
    };
    assert_eq!(route(&ssh, &live), Some(Route::Ssh("host".into())));
    assert_eq!(route(&ConnectTarget::Local, &live), Some(Route::Local));
    for target in [
        ConnectTarget::Wsl {
            distro: "Ubuntu".into(),
            session: "default".into(),
        },
        ConnectTarget::Socket("/tmp/herdr.sock".into()),
    ] {
        assert_eq!(route(&target, &live), None, "{target:?}");
    }
    // A connection that has gone answers nothing, local or remote.
    assert_eq!(route(&ssh, &LiveState::default()), None);
}

#[test]
fn a_remote_daemon_is_not_local() {
    let live = connected(true);
    for target in [
        ConnectTarget::Ssh {
            target: "host".into(),
            session: "default".into(),
        },
        ConnectTarget::Wsl {
            distro: "Ubuntu".into(),
            session: "default".into(),
        },
        // A socket by path may be forwarded from another host.
        ConnectTarget::Socket("/tmp/herdr.sock".into()),
    ] {
        assert!(!local(&target, &live), "{target:?}");
    }
}

#[test]
fn only_an_owned_connected_local_daemon_is_local() {
    assert!(local(&ConnectTarget::Local, &connected(true)));
    assert!(local(
        &ConnectTarget::Session {
            name: "work".into(),
            development: false
        },
        &connected(true)
    ));
    // Windows has no peer check; its chat pipe's owner is checked instead.
    assert_eq!(
        local(&ConnectTarget::Local, &connected(false)),
        cfg!(windows)
    );
    assert!(!local(&ConnectTarget::Local, &LiveState::default()));
}
