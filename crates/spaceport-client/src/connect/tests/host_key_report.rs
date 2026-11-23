//! What a worker reports about the host key its bridge was and was not asked
//! to check, so a window can show the connection as unverified.
use super::*;

#[test]
fn a_bridge_that_skipped_the_host_key_is_reported_ahead_of_its_handshake() {
    let (tx, rx) = bounded(1);
    let stop = AtomicBool::new(false);
    report_host_keys(Some(ssh::HostKeys::Skip), &tx, &stop).unwrap();
    assert!(matches!(rx.try_recv(), Ok(ClientEvent::HostKeysUnverified)));
    assert!(rx.try_recv().is_err());
}

#[test]
fn a_checked_host_and_a_transport_without_a_host_key_report_nothing() {
    let (tx, rx) = bounded(1);
    let stop = AtomicBool::new(false);
    for host_keys in [Some(ssh::HostKeys::Verify), None] {
        report_host_keys(host_keys, &tx, &stop).unwrap();
    }
    assert!(rx.try_recv().is_err());
}
