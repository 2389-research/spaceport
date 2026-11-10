// ABOUTME: Proves chat presentation and disconnect only affect the view and attach link.
// ABOUTME: Uses isolated protocol peers to detect accidental agent or terminal input.
use super::*;
use crate::{connection::ConnectionBridge, state::ConnectionStatus, terminal::InputTarget};
use gpui::TestAppContext;
use sheprd_client::{
    ConnectTarget,
    protocol::{ClientMessage as HerdrMessage, ClientPaneInputEvent},
};

#[gpui::test]
fn presentation_and_disconnect_send_no_agent_or_terminal_commands(cx: &mut TestAppContext) {
    let (view, mut runtime, cx) = overlay::open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut adapter = overlay::attach_with(
        &view,
        &program,
        snapshot(cursor(1, 0), vec![agent(1, "Ready")]),
        cx,
    );
    for raw in [true, false, true, false] {
        cx.update(|window, cx| {
            view.update(cx, |view, cx| view.set_chat_raw("w1:p1", raw, window, cx));
        });
        overlay::poll(&view, &program, cx);
        view.read_with(cx, |view, _| {
            assert_eq!(view.chats.shown("w1:p1"), !raw);
            assert!(view.chats.starting.is_none());
        });
    }
    view.update(cx, |view, _| {
        view.live.status = ConnectionStatus::Disconnected
    });
    overlay::poll(&view, &program, cx);
    view.read_with(cx, |view, _| {
        assert!(view.chats.pane("w1:p1").is_none());
        assert!(view.chats.starting.is_none());
        ConnectionBridge::send_input(
            view.endpoints[0].connection.handle.as_ref().unwrap(),
            &view.live.snapshot.as_ref().unwrap().boot_id,
            &InputTarget::Pane("w1:p1".into()),
            ClientPaneInputEvent::TextCommit("sentinel".into()),
        )
        .unwrap();
    });
    // The link closed without a prompt, cancellation, or permission answer.
    assert_eq!(read(&mut adapter), None);
    // The sentinel follows all queued view work on the ordered runtime wire.
    loop {
        match runtime.receive() {
            HerdrMessage::ClientShellPaneInput { events, .. } => {
                assert_eq!(
                    events,
                    [ClientPaneInputEvent::TextCommit("sentinel".into())]
                );
                break;
            }
            HerdrMessage::ClientShellEndpointRequest { request, .. } => {
                panic!("presentation sent a runtime request: {request}");
            }
            HerdrMessage::Input { .. } | HerdrMessage::ClientShellPopupInput { .. } => {
                panic!("presentation sent terminal input");
            }
            _ => {}
        }
    }
}
