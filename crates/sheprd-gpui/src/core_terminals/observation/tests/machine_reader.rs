use super::*;
use std::time::Duration;

/// The machine reader no longer naps between passes: replacing a document
/// request wakes it at once, and a wake already pending is not queued twice.
/// Both receives are lookups of a wake already sent, never a wait.
#[test]
fn a_replaced_document_request_wakes_the_machine_reader() {
    let (wake, woken) = mpsc::sync_channel(1);
    let requests = RemoteRequests {
        latest: Arc::new(Mutex::new(None)),
        wake,
    };
    let mut remote = Remote::default();
    let connected = [machine("stable-a", "lab", "connected")];
    let first = remote
        .select(1, 1, Some("stable-a".into()), Some(&connected))
        .unwrap();
    requests.replace(first.clone());
    assert!(
        woken.recv_timeout(Duration::ZERO).is_ok(),
        "a request did not wake the reader"
    );
    assert_eq!(take_latest(&requests.latest), Some(first));
    assert!(
        woken.recv_timeout(Duration::ZERO).is_err(),
        "a taken request left a wake behind"
    );

    let second = remote
        .select(2, 2, Some("stable-a".into()), Some(&connected))
        .unwrap();
    requests.replace(second.clone());
    requests.replace(second.clone());
    assert!(woken.recv_timeout(Duration::ZERO).is_ok());
    assert!(
        woken.recv_timeout(Duration::ZERO).is_err(),
        "a pending wake was queued twice"
    );
    assert_eq!(take_latest(&requests.latest), Some(second));
}
