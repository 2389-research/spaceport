//! Up and Down step through the session's earlier prompts, whichever client
//! sent them, including those the session had before the chat attached, and
//! one this window sent that the agent has yet to echo.

use super::{
    overlay::{attach_with, open},
    *,
};
use crate::SheprdWindow;
use gpui::{Entity, TestAppContext, VisualTestContext};
use sheprd_chat_protocol::Command;
use sheprd_client::ConnectTarget;

fn user(id: u64, text: &str) -> Entry {
    Entry {
        id: EntryId(id),
        kind: EntryKind::User {
            text: text.into(),
            files: 0,
        },
    }
}

fn earlier() -> ServerMessage {
    let mut transcript = Transcript::new("Claude");
    transcript.status = Status::Idle;
    transcript.entries = vec![
        user(1, "first"),
        agent(2, "Done."),
        user(3, "second"),
        agent(4, "Done."),
    ];
    ServerMessage::Snapshot {
        cursor: cursor(1, 0),
        transcript: Box::new(transcript),
    }
}

fn prompt_text(stream: &mut Peer) -> String {
    match read(stream) {
        Some(ClientMessage::Command {
            command: Command::Prompt { text, .. },
            ..
        }) => text,
        other => panic!("a prompt, got {other:?}"),
    }
}

#[gpui::test]
fn up_and_down_step_through_the_sessions_prompts(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, earlier(), cx);

    cx.simulate_keystrokes("up up");
    cx.simulate_keystrokes("enter");
    assert_eq!(prompt_text(&mut stream), "first");

    cx.simulate_keystrokes("up up up down");
    cx.simulate_keystrokes("enter");
    assert_eq!(prompt_text(&mut stream), "second");
}

fn composer_text(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) -> String {
    view.read_with(cx, |view, cx| {
        let pane = view.chats.pane("w1:p1").unwrap();
        pane.composer.clone().unwrap().read(cx).text().to_owned()
    })
}

#[gpui::test]
fn a_prompt_the_window_sent_is_recalled_before_the_agent_echoes_it(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    cx.simulate_input("hello there");
    cx.simulate_keystrokes("enter");
    assert_eq!(prompt_text(&mut stream), "hello there");
    cx.simulate_keystrokes("up");
    assert_eq!(composer_text(&view, cx), "hello there");
}

#[gpui::test]
fn a_prompt_the_window_could_not_send_stays_out_of_history(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let _stream = attach_with(&view, &program, snapshot(cursor(1, 0), vec![]), cx);
    // A full queue refuses the next command, as a stalled link's does.
    view.update(cx, |view, _| {
        let pane = view.chats.pane_mut("w1:p1").unwrap();
        for _ in 0..MAX_PENDING {
            pane.session.submit(Command::Cancel).unwrap();
        }
    });
    cx.simulate_input("keep me");
    cx.simulate_keystrokes("enter");
    let error = view.read_with(cx, |view, _| {
        view.chats.pane("w1:p1").unwrap().error.clone()
    });
    assert!(error.is_some(), "the send is refused");
    assert_eq!(
        composer_text(&view, cx),
        "keep me",
        "the prompt stays in the composer"
    );
    // Nothing recalls it: it never reached the agent.
    view.update(cx, |view, cx| {
        let input = view.chats.pane("w1:p1").unwrap().composer.clone().unwrap();
        input.update(cx, |input, cx| input.set_text_at("", 0, cx));
    });
    cx.simulate_keystrokes("up");
    assert_eq!(
        composer_text(&view, cx),
        "",
        "a refused prompt is not recalled"
    );
}
