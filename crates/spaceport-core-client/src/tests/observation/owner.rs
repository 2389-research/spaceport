// ABOUTME: Check the connection lifetime required by a Core event subscription.
// ABOUTME: Keep ownership tests independent of event delivery timing.
use super::*;

#[test]
fn dropping_the_backend_ends_its_watch_even_while_the_peer_stays_open() {
    let (client, mut core) = connected();
    let backend = CoreBackend::new(client, 1000);
    let mut changes = watching(&backend, &mut core);

    drop(backend);

    assert!(matches!(
        changes.next_batch(Duration::from_secs(5)),
        Err(crate::Error::Lost)
    ));
}
