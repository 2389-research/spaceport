// ABOUTME: Checks the shutdown card and its explicit restart action.
// ABOUTME: Keeps the restart click on an isolated fixture target.
use super::*;
use gpui::Modifiers;
use sheprd_client::{ClientEvent, ConnectTarget, protocol::ServerMessage};

#[gpui::test]
fn a_shutdown_shows_an_explicit_start_button(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        let endpoint = &mut view.endpoints[0];
        endpoint.connection.target = ConnectTarget::Local;
        {
            let mut inbox = endpoint.connection.inbox.lock().unwrap();
            inbox.apply(ClientEvent::Message(ServerMessage::ServerShutdown {
                reason: None,
            }));
            inbox.apply(ClientEvent::Disconnected {
                reason: "fixture shutdown".into(),
                ssh: None,
            });
        }
        view.poll_endpoints(cx);
        assert!(view.endpoints[0].herdr_stopped());
        // A click must not reach or start the personal daemon.
        view.endpoints[0].connection.target =
            ConnectTarget::Socket("/unused-shutdown-test.sock".into());
        cx.notify();
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("herdr-stopped").is_some());
    assert!(cx.debug_bounds("herdr-stopped-note").is_some());
    assert!(cx.debug_bounds("reconnect-now").is_none());
    let generation = view.read_with(cx, |view, _| view.endpoints[0].generation);
    let start = cx.debug_bounds("start-herdr").unwrap();
    cx.simulate_click(start.center(), Modifiers::default());
    view.read_with(cx, |view, _| {
        assert!(view.endpoints[0].generation > generation)
    });
}
