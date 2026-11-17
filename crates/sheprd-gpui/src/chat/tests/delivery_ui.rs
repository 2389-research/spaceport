// ABOUTME: Exercises prompt delivery from the composer through a real endpoint.
// ABOUTME: Shows conversation changes without replaying an old command.
use super::{
    overlay::{attach_with, open, poll},
    *,
};
use crate::chat::Refusal;
use gpui::TestAppContext;
use sheprd_client::ConnectTarget;

#[gpui::test]
fn a_replaced_conversation_shows_delivery_uncertainty_and_accepts_a_fresh_prompt(
    cx: &mut TestAppContext,
) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, snapshot(cursor(1, 0), Vec::new()), cx);
    cx.simulate_input("Fixture prompt before replacement");
    cx.simulate_keystrokes("enter");
    let Some(ClientMessage::Command { id: previous, .. }) = read(&mut stream) else {
        panic!("prompt")
    };
    view.read_with(cx, |view, cx| {
        assert_eq!(
            view.chats
                .panes()
                .next()
                .unwrap()
                .composer
                .as_ref()
                .unwrap()
                .read(cx)
                .text(),
            ""
        );
    });
    send(
        &mut stream,
        &snapshot(cursor(2, 0), vec![agent(1, "Replacement fixture")]),
    );
    wait_until(|| {
        poll(&view, &program, cx);
        cx.debug_bounds("chat-problem").is_some()
    });
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.chats.panes().next().unwrap().session.rejection(),
            Some(Refusal::SessionChanged)
        );
    });
    send(
        &mut stream,
        &ServerMessage::Ack {
            id: previous,
            rejected: None,
        },
    );
    cx.simulate_input("Fresh fixture prompt");
    cx.simulate_keystrokes("enter");
    let Some(ClientMessage::Command { command, id }) = read(&mut stream) else {
        panic!("fresh prompt")
    };
    assert_ne!(id, previous);
    assert!(
        matches!(command, sheprd_chat_protocol::Command::Prompt { text, .. } if text == "Fresh fixture prompt")
    );
    poll(&view, &program, cx);
    assert!(cx.debug_bounds("chat-problem").is_none());
}
