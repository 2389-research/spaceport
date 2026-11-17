// ABOUTME: Checks tool status rendering across transcript lifecycle changes.
// ABOUTME: Exercises the line cache through real attach message reduction.
use super::*;
use sheprd_chat_protocol::{Tool, ToolKind, ToolStatus};

fn working() -> Transcript {
    Transcript {
        status: Status::Working,
        entries: vec![Entry {
            id: EntryId(1),
            kind: EntryKind::Tool(Tool {
                call_id: "running-call".into(),
                content: Vec::new(),
                kind: ToolKind::Execute,
                locations: Vec::new(),
                parent: None,
                started_ms: Some(1),
                status: ToolStatus::InProgress,
                title: "fixture tool".into(),
            }),
        }],
        ..Transcript::new("fixture")
    }
}

fn mark(view: &mut View) -> String {
    view.lines(60)[0].spans[0].content.to_string()
}

#[test]
fn cached_tool_stops_when_the_session_or_turn_stops() {
    for status in [
        Status::Idle,
        Status::Failed {
            message: "fixture".into(),
        },
        Status::Exited {
            message: "fixture".into(),
        },
    ] {
        let mut view = View::default();
        let cursor = snapshot(&mut view, working());
        assert_eq!(mark(&mut view), "◐ ");
        view.receive(ServerMessage::Change {
            cursor: Cursor { seq: 2, ..cursor },
            change: Change::Status { status },
        });
        assert_eq!(mark(&mut view), "■ ");
    }
}

#[test]
fn a_later_prompt_does_not_revive_an_older_unfinished_tool() {
    let mut view = View::default();
    let cursor = snapshot(&mut view, working());
    assert_eq!(mark(&mut view), "◐ ");
    view.receive(ServerMessage::Change {
        cursor: Cursor { seq: 2, ..cursor },
        change: Change::Push {
            entry: Entry {
                id: EntryId(2),
                kind: EntryKind::User {
                    text: "next".into(),
                    files: 0,
                },
            },
        },
    });
    assert_eq!(mark(&mut view), "■ ");
}
