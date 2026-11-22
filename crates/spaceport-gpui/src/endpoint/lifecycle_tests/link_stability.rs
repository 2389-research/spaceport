// ABOUTME: Checks retry stability when a stopped handle still has a fresh snapshot.
// ABOUTME: Forces that ordering without racing the transport event reader.
use super::*;

#[test]
fn a_stopped_handle_cannot_finish_the_stability_period() {
    let (mut endpoint, _server) = connected_endpoint(LOCAL);
    endpoint.attempts = 8;
    endpoint.clear_stability_for_test();
    let now = Instant::now();
    endpoint.poll(now);
    // The event reader retains the old inbox. This fresh update still says
    // connected, while the handle already proves the transport was stopped.
    let mut state = endpoint.live.clone();
    state.dirty = true;
    endpoint.connection.inbox = Arc::new(Mutex::new(state));
    endpoint.connection.handle.as_ref().unwrap().disconnect();
    endpoint.poll(now + STABLE_CONNECTION_PERIOD);
    assert_eq!(endpoint.attempts, 8);
    assert_eq!(
        endpoint.retry_at(),
        now + STABLE_CONNECTION_PERIOD + MAX_RETRY_DELAY
    );
}
