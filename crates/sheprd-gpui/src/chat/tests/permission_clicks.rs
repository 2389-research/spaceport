// ABOUTME: Exercises permission buttons through rendered pointer events and real frames.
// ABOUTME: Keeps a double-click from answering a newly displayed permission.
use super::{
    overlay::{attach_with, open, poll},
    *,
};
use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, TestAppContext};
use sheprd_chat_protocol::{Command, Permission, PermissionId, PermissionKind, PermissionOption};
use sheprd_client::ConnectTarget;

fn permission(id: u64) -> Permission {
    Permission {
        call_id: None,
        id: PermissionId(id),
        options: vec![PermissionOption {
            id: "allow".into(),
            kind: PermissionKind::AllowOnce,
            name: "Allow once".into(),
        }],
        title: format!("Request {id}"),
    }
}

fn click(cx: &mut gpui::VisualTestContext, count: usize) {
    let position = cx.debug_bounds("chat-answer-0").unwrap().center();
    cx.simulate_event(MouseDownEvent {
        position,
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: count,
        first_mouse: false,
    });
    cx.simulate_event(MouseUpEvent {
        position,
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: count,
    });
}

fn command(stream: &mut Peer) -> (sheprd_chat_protocol::CommandId, Command) {
    match read(stream).unwrap() {
        ClientMessage::Command { id, command } => (id, command),
        other => panic!("expected command, got {other:?}"),
    }
}

#[gpui::test]
fn a_second_click_cannot_answer_the_next_permission(cx: &mut TestAppContext) {
    let (view, _daemon, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut transcript = Transcript::new("Claude");
    transcript.permissions = vec![permission(1)];
    let mut stream = attach_with(
        &view,
        &program,
        ServerMessage::Snapshot {
            cursor: cursor(1, 0),
            transcript: Box::new(transcript),
        },
        cx,
    );
    click(cx, 1);
    let (id, answered) = command(&mut stream);
    assert_eq!(
        answered,
        Command::Answer {
            option: "allow".into(),
            permission: PermissionId(1)
        }
    );
    send(&mut stream, &ServerMessage::Ack { id, rejected: None });
    send(
        &mut stream,
        &change(
            Change::PermissionResolved {
                id: PermissionId(1),
            },
            cursor(1, 1),
        ),
    );
    send(
        &mut stream,
        &change(
            Change::PermissionAsked {
                permission: permission(2),
            },
            cursor(1, 2),
        ),
    );
    wait_until(|| {
        poll(&view, &program, cx);
        view.read_with(cx, |view, _| {
            view.chats
                .panes()
                .next()
                .unwrap()
                .session
                .transcript()
                .permissions
                .first()
                .is_some_and(|p| p.id == PermissionId(2))
        })
    });
    click(cx, 2);
    // A later command is a FIFO barrier, avoiding an absence assertion based on sleeping.
    view.update(cx, |view, cx| {
        let pane_id = view.chats.panes().next().unwrap().target.pane_id.clone();
        view.chat_send(&pane_id, Command::Cancel, cx);
        assert!(view.chats.pane(&pane_id).unwrap().error.is_none());
    });
    assert_eq!(command(&mut stream).1, Command::Cancel);
    click(cx, 1);
    assert_eq!(
        command(&mut stream).1,
        Command::Answer {
            option: "allow".into(),
            permission: PermissionId(2)
        }
    );
}
