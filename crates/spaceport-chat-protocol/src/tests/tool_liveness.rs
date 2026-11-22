// ABOUTME: Verifies tool liveness from authoritative transcript state.
// ABOUTME: Covers stopped turns, session failure, history, and background activities.
use super::*;

fn call(entry: &Entry) -> &Tool {
    let EntryKind::Tool(tool) = &entry.kind else {
        panic!("tool fixture")
    };
    tool
}

#[test]
fn only_unfinished_calls_in_the_current_working_turn_are_live() {
    let mut transcript = Transcript {
        status: Status::Working,
        entries: vec![tool(1, ToolStatus::InProgress)],
        ..Transcript::new("fixture")
    };
    assert!(transcript.tool_live(call(&transcript.entries[0])));
    for status in [
        ToolStatus::Pending,
        ToolStatus::InProgress,
        ToolStatus::Completed,
        ToolStatus::Failed,
    ] {
        transcript.entries[0] = tool(1, status);
        assert_eq!(
            transcript.tool_live(call(&transcript.entries[0])),
            matches!(status, ToolStatus::Pending | ToolStatus::InProgress)
        );
    }
    transcript.entries[0] = tool(1, ToolStatus::InProgress);
    transcript.entries.push(user(2, "next"));
    assert!(!transcript.tool_live(call(&transcript.entries[0])));
    for status in [
        Status::Starting,
        Status::Idle,
        Status::Failed {
            message: "fixture".into(),
        },
        Status::Exited {
            message: "fixture".into(),
        },
    ] {
        transcript.status = status;
        transcript.entries.truncate(1);
        assert!(!transcript.tool_live(call(&transcript.entries[0])));
    }
}

#[test]
fn background_work_can_outlive_a_turn_but_not_the_session() {
    let mut transcript = Transcript {
        status: Status::Idle,
        entries: vec![tool(1, ToolStatus::Completed)],
        activities: vec![Activity {
            call_id: "call-1".into(),
            detail: None,
            finished_ms: None,
            kind: ActivityKind::Shell,
            label: "fixture".into(),
            started_ms: 1,
            state: ActivityState::Running,
        }],
        ..Transcript::new("fixture")
    };
    assert!(transcript.tool_live(call(&transcript.entries[0])));
    transcript.entries.push(user(2, "next"));
    assert!(transcript.tool_live(call(&transcript.entries[0])));
    for state in [
        ActivityState::Finished,
        ActivityState::Killed,
        ActivityState::Failed,
    ] {
        transcript.activities[0].state = state;
        assert!(!transcript.tool_live(call(&transcript.entries[0])));
    }
    transcript.activities[0].state = ActivityState::Running;
    for status in [
        Status::Failed {
            message: "fixture".into(),
        },
        Status::Exited {
            message: "fixture".into(),
        },
    ] {
        transcript.status = status;
        assert!(!transcript.tool_live(call(&transcript.entries[0])));
    }
}

#[test]
fn a_nested_step_follows_its_parent_activity() {
    let mut entry = tool(1, ToolStatus::InProgress);
    if let EntryKind::Tool(tool) = &mut entry.kind {
        tool.parent = Some("parent-call".into());
    }
    let mut transcript = Transcript {
        status: Status::Idle,
        entries: vec![entry],
        activities: vec![Activity {
            call_id: "parent-call".into(),
            detail: None,
            finished_ms: None,
            kind: ActivityKind::Subagent,
            label: "fixture".into(),
            started_ms: 1,
            state: ActivityState::Running,
        }],
        ..Transcript::new("fixture")
    };
    assert!(transcript.tool_live(call(&transcript.entries[0])));
    transcript.status = Status::Working;
    transcript.entries.push(user(2, "another turn"));
    assert!(transcript.tool_live(call(&transcript.entries[0])));
    transcript.entries.pop();
    for state in [
        ActivityState::Finished,
        ActivityState::Killed,
        ActivityState::Failed,
    ] {
        transcript.activities[0].state = state;
        assert!(!transcript.tool_live(call(&transcript.entries[0])));
    }
    transcript.activities[0].state = ActivityState::Running;
    if let EntryKind::Tool(tool) = &mut transcript.entries[0].kind {
        tool.status = ToolStatus::Completed;
    }
    assert!(!transcript.tool_live(call(&transcript.entries[0])));
}
