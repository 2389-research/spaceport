// ABOUTME: Tests bounded command retries through real isolated chat endpoints.
// ABOUTME: Preserves writer outcomes across EOF and stream reducer failures.
use super::*;
use crate::chat::Refusal;

#[test]
fn two_real_writes_exhaust_retries_even_when_a_reducer_error_ends_the_link() {
    let mut harness = Harness::new();
    let mut stream = harness.attach(cursor(1, 0), Vec::new());
    harness
        .chats
        .pane_mut("p1")
        .unwrap()
        .send(Command::Cancel)
        .unwrap();
    let original = read(&mut stream).unwrap();
    for _ in 0..2 {
        // The peer received the frame before sending a malformed stream position.
        send(&mut stream, &change(append(5, "gap"), cursor(1, 2)));
        harness.poll_until(|chats| !chats.pane("p1").unwrap().has_link());
        harness.reconcile(vec![target("p1", A)]);
        let (next, _) = harness.program.accept();
        stream = next;
        send(&mut stream, &welcome());
        send(&mut stream, &snapshot(cursor(1, 0), Vec::new()));
        harness.poll_until(|chats| chats.pane("p1").unwrap().attached());
        if harness
            .chats
            .pane("p1")
            .unwrap()
            .session
            .rejection()
            .is_none()
        {
            assert_eq!(read(&mut stream), Some(original.clone()));
        }
    }
    assert_eq!(
        harness.chats.pane("p1").unwrap().session.rejection(),
        Some(Refusal::DeliveryUnconfirmed)
    );
    harness
        .chats
        .pane_mut("p1")
        .unwrap()
        .send(Command::SetAutoAccept { on: true })
        .unwrap();
    assert!(matches!(
        read(&mut stream),
        Some(ClientMessage::Command {
            command: Command::SetAutoAccept { on: true },
            ..
        })
    ));
}

#[test]
fn a_reconnect_to_a_different_epoch_never_replays_a_written_prompt() {
    let mut harness = Harness::new();
    let mut stream = harness.attach(cursor(1, 0), Vec::new());
    harness
        .chats
        .pane_mut("p1")
        .unwrap()
        .send(Command::Prompt {
            text: "Old conversation fixture".into(),
            attachments: Vec::new(),
        })
        .unwrap();
    let original = read(&mut stream).unwrap();
    drop(stream);
    harness.poll_until(|chats| !chats.pane("p1").unwrap().has_link());
    harness.later();
    harness.reconcile(vec![target("p1", A)]);
    let (mut stream, greeting) = harness.program.accept();
    assert_eq!(greeting, hello(Some(cursor(1, 0))));
    send(&mut stream, &welcome());
    send(&mut stream, &snapshot(cursor(2, 0), Vec::new()));
    harness.poll_until(|chats| chats.pane("p1").unwrap().attached());
    assert_eq!(
        harness.chats.pane("p1").unwrap().session.rejection(),
        Some(Refusal::SessionChanged)
    );
    harness
        .chats
        .pane_mut("p1")
        .unwrap()
        .send(Command::Cancel)
        .unwrap();
    let next = read(&mut stream).unwrap();
    assert_ne!(next, original);
    assert!(matches!(
        next,
        ClientMessage::Command {
            command: Command::Cancel,
            ..
        }
    ));
}
