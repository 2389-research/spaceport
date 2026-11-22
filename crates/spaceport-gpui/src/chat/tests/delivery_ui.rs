// ABOUTME: Exercises prompt delivery from the composer through a real endpoint.
// ABOUTME: Shows conversation changes without replaying an old command.
use super::{
    overlay::{attach_with, open, poll, reconnect},
    *,
};
use crate::chat::Refusal;
use gpui::TestAppContext;
use spaceport_client::ConnectTarget;

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
        matches!(command, spaceport_chat_protocol::Command::Prompt { text, .. } if text == "Fresh fixture prompt")
    );
    poll(&view, &program, cx);
    assert!(cx.debug_bounds("chat-problem").is_none());
}

#[gpui::test]
fn a_reconnect_repeats_the_unconfirmed_prompt_with_its_id_and_mints_nothing(
    cx: &mut TestAppContext,
) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(
        &view,
        &program,
        snapshot(cursor(1, 0), vec![agent(1, "Ready.")]),
        cx,
    );
    cx.simulate_input("Fixture prompt across a reconnect");
    cx.simulate_keystrokes("enter");
    let Some(ClientMessage::Command { command, id }) = read(&mut stream) else {
        panic!("prompt")
    };
    // The program took the write and never acknowledged it: dropping its end
    // leaves the window with an unknown outcome in the same conversation.
    drop(stream);
    let (mut stream, greeting) = reconnect(&view, &program, cx);
    assert_eq!(greeting, hello(Some(cursor(1, 0))));
    send(&mut stream, &welcome());
    send(
        &mut stream,
        &snapshot(cursor(1, 0), vec![agent(1, "Ready.")]),
    );
    // The repeat waits for the new connection to confirm the conversation's
    // epoch, which the snapshot does.
    wait_until(|| {
        poll(&view, &program, cx);
        view.read_with(cx, |view, _| {
            view.chats.pane("w1:p1").unwrap().session.live()
        })
    });
    // The one repeat carries the id and body the window already wrote, so the
    // program can tell it from a new prompt. Nothing else is sent for it, and
    // the following reconnect in this epoch leaves it alone.
    assert_eq!(
        read(&mut stream),
        Some(ClientMessage::Command { command, id })
    );
    send(&mut stream, &ServerMessage::Ack { id, rejected: None });
    poll(&view, &program, cx);
    view.update(cx, |view, _| {
        let pane = view.chats.pane_mut("w1:p1").unwrap();
        assert_eq!(pane.session.rejection(), None);
        assert!(pane.session.resend().is_empty());
    });
}

#[gpui::test]
fn a_reconnect_into_a_replaced_conversation_sends_nothing(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(
        &view,
        &program,
        snapshot(cursor(1, 0), vec![agent(1, "Ready.")]),
        cx,
    );
    cx.simulate_input("Fixture prompt before the program restarts");
    cx.simulate_keystrokes("enter");
    let Some(ClientMessage::Command { id: previous, .. }) = read(&mut stream) else {
        panic!("prompt")
    };
    drop(stream);
    let (mut stream, greeting) = reconnect(&view, &program, cx);
    assert_eq!(greeting, hello(Some(cursor(1, 0))));
    send(&mut stream, &welcome());
    // A restarted program answers for a new conversation, so the prompt that
    // conversation never acknowledged must not be sent into it.
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
    // The first command on the new conversation is the one the user sends
    // now, with an id of its own.
    cx.simulate_input("Fresh fixture prompt");
    cx.simulate_keystrokes("enter");
    let Some(ClientMessage::Command { id, .. }) = read(&mut stream) else {
        panic!("fresh prompt")
    };
    assert_ne!(id, previous);
}
