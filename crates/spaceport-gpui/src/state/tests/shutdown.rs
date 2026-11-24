// ABOUTME: Makes deliberate shutdown visible before its following disconnect.
// ABOUTME: Requires the state reducer to invalidate the whole window for shutdown advice.
use super::*;

#[test]
fn shutdown_requests_a_window_update_before_disconnect() {
    let state = LiveState::default();
    let mut stopped = state.clone();
    stopped.dirty = false;
    stopped.apply(
        ClientEvent::Message(ServerMessage::ServerShutdown { reason: None }),
        &labels(),
    );
    assert!(stopped.dirty, "the shutdown message must reach the UI");
    assert!(
        !state.only_surface_changed(&stopped),
        "shutdown advice changes window chrome"
    );
}
