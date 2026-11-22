// ABOUTME: Checks actual Timeline paint after tool lifecycle changes.
// ABOUTME: Keeps stopped and historical calls from painting a running marker.
use super::{
    overlay::{attach_with, open, poll},
    *,
};
use gpui::{TestAppContext, rgb};
use spaceport_chat_protocol::{Change, Tool, ToolKind, ToolStatus};
use spaceport_client::ConnectTarget;

#[gpui::test]
fn a_stopped_timeline_tool_loses_its_running_marker(cx: &mut TestAppContext) {
    let (view, _peer, cx) = open(cx, ConnectTarget::Local);
    let program = Program::new();
    let mut stream = attach_with(
        &view,
        &program,
        ServerMessage::Snapshot {
            cursor: cursor(1, 0),
            transcript: Box::new(Transcript {
                status: Status::Working,
                entries: vec![
                    agent(1, "Inspecting."),
                    Entry {
                        id: EntryId(2),
                        kind: EntryKind::Tool(Tool {
                            call_id: "fixture".into(),
                            content: Vec::new(),
                            kind: ToolKind::Execute,
                            locations: Vec::new(),
                            parent: None,
                            started_ms: None,
                            status: ToolStatus::InProgress,
                            title: "fixture tool".into(),
                        }),
                    },
                ],
                ..Transcript::new("fixture")
            }),
        },
        cx,
    );
    assert!(cx.debug_bounds("chat-dot-2").is_some());
    let yellow = cx.read(|cx| view::Hues::of(&view.read(cx).theme).yellow);
    let running = |cx: &mut gpui::VisualTestContext| {
        cx.update(|window, _| {
            window
                .painted_quads()
                .iter()
                .any(|quad| quad.border_color == rgb(yellow).into())
        })
    };
    assert!(running(cx));
    send(
        &mut stream,
        &ServerMessage::Change {
            cursor: cursor(1, 1),
            change: Change::Status {
                status: Status::Idle,
            },
        },
    );
    wait_until(|| {
        poll(&view, &program, cx);
        !running(cx)
    });
    assert!(cx.debug_bounds("chat-dot-2").is_some());
}
