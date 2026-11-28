//! A local endpoint whose Herdr cannot be found waits the longest retry delay,
//! says so to commands, and forgets it once a connection succeeds.
use super::*;

const MISSING: &str =
    "Could not talk to the daemon: Herdr is not installed: No such file or directory (os error 2)";

/// Stands in for the connector finding no `herdr` to start: the bridge marks
/// the inbox before the worker reports the failure and stops.
fn lose_herdr(endpoint: &mut Endpoint) {
    let mut state = endpoint.connection.inbox.lock().unwrap();
    state.missing_installation = true;
    state.apply(
        ClientEvent::Disconnected {
            reason: MISSING.into(),
            ssh: None,
        },
        crate::state::describe,
        crate::state::localize,
    );
}

#[test]
fn a_missing_herdr_waits_the_longest_retry_delay() {
    let (mut endpoint, _server) = connected_endpoint(LOCAL);
    lose_herdr(&mut endpoint);
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    let now = Instant::now();
    // The inbox may be busy for a poll; the state still lands on a later one.
    wait_until(|| {
        endpoint.poll(now);
        endpoint.live().missing_installation
    });
    assert_eq!(endpoint.retry_at(), now + MAX_RETRY_DELAY);
    assert_eq!(endpoint.outage(), Some(MISSING));
    assert!(endpoint.herdr_missing());
    assert_eq!(endpoint.status(), "Herdr not installed");
}

/// The failure can reach the inbox a poll after the stop it explains.
#[test]
fn a_missing_herdr_seen_after_its_stop_still_waits_the_longest_delay() {
    let (mut endpoint, _server) = connected_endpoint(LOCAL);
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    let dropped = Instant::now();
    endpoint.poll(dropped);
    assert_eq!(endpoint.retry_at(), dropped + endpoint.retry_delay());

    lose_herdr(&mut endpoint);
    let arrived = dropped + Duration::from_millis(16);
    wait_until(|| {
        endpoint.poll(arrived);
        endpoint.live().missing_installation
    });
    assert_eq!(endpoint.outage(), Some(MISSING));
    assert_eq!(endpoint.retry_at(), arrived + MAX_RETRY_DELAY);
    assert!(endpoint.herdr_missing());
}

/// Each retry starts from a fresh state that has not failed yet; the endpoint
/// keeps saying Herdr is missing until a connection proves otherwise.
#[test]
fn a_missing_herdr_outlives_retries_until_a_connection_succeeds() {
    let (mut endpoint, _server) = connected_endpoint(LOCAL);
    lose_herdr(&mut endpoint);
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.live().missing_installation
    });
    endpoint.connect(ConnectOptions::default(), false);
    assert!(!endpoint.live().missing_installation, "the fresh attempt");
    assert!(endpoint.herdr_missing());
    assert_eq!(endpoint.status(), "Herdr not installed");

    // A later attempt's handshake succeeds: Herdr was installed meanwhile.
    // A bridge with no worker keeps the stand-in state from being raced.
    endpoint.connection = ConnectionBridge::new(endpoint.connection.target.clone());
    {
        let mut state = endpoint.connection.inbox.lock().unwrap();
        state.status = ConnectionStatus::AwaitingSnapshot;
        state.dirty = true;
    }
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.live().status.is_connected()
    });
    assert!(!endpoint.herdr_missing());
    assert_ne!(endpoint.status(), "Herdr not installed");
}

/// Installing a Herdr too old for this app turns "not installed" into
/// "update needed" at once: the refused handshake proves Herdr was found.
#[test]
fn a_version_refusal_replaces_a_missing_herdr() {
    let (mut endpoint, _server) = connected_endpoint(LOCAL);
    lose_herdr(&mut endpoint);
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.live().missing_installation
    });
    assert!(endpoint.herdr_missing());

    // A bridge with no worker keeps the stand-in state from being raced.
    endpoint.connection = ConnectionBridge::new(endpoint.connection.target.clone());
    {
        let mut state = endpoint.connection.inbox.lock().unwrap();
        state.version_mismatch = Some(VersionMismatch::DaemonOutdated {
            server_version: Some("0.8.2".into()),
        });
        state.dirty = true;
    }
    wait_until(|| {
        endpoint.poll(Instant::now());
        endpoint.live().version_mismatch.is_some()
    });
    assert!(!endpoint.herdr_missing(), "an outdated Herdr is installed");
    assert_eq!(endpoint.status(), "Herdr update needed");
}
