// ABOUTME: Checks chat behavior against synthetic transcript fixtures.
// ABOUTME: Keeps lifecycle and rendering expectations explicit at the test boundary.
use super::*;
use spaceport_chat_protocol::{Activity, ActivityKind, EntryId, Status};

#[core::prelude::v1::test]
fn a_card_says_how_what_its_call_started_stands() {
    // A background shell's call is done, but the shell runs on.
    assert_eq!(
        state(Some(ActivityState::Running), ToolStatus::Completed),
        ToolState::Running
    );
    assert_eq!(
        state(Some(ActivityState::Killed), ToolStatus::Completed),
        ToolState::Stopped
    );
    assert_eq!(
        state(Some(ActivityState::Failed), ToolStatus::Completed),
        ToolState::Failed
    );
    assert_eq!(
        state(Some(ActivityState::Finished), ToolStatus::InProgress),
        ToolState::Done
    );
    // Without one, the call's own status.
    assert_eq!(state(None, ToolStatus::Pending), ToolState::Pending);
    assert_eq!(state(None, ToolStatus::InProgress), ToolState::Running);
    assert_eq!(state(None, ToolStatus::Failed), ToolState::Failed);
}

#[core::prelude::v1::test]
fn a_sub_agent_counts_its_steps() {
    let plain = |text: String| text.replace(['\u{2068}', '\u{2069}'], "");
    assert_eq!(plain(steps_label(1)), "1 step");
    assert_eq!(plain(steps_label(12)), "12 steps");
    assert_eq!(tool_kind(ToolKind::Subagent), "Sub-agent");
}

fn state(activity: Option<ActivityState>, status: ToolStatus) -> ToolState {
    let tool = Tool {
        call_id: "fixture".into(),
        content: Vec::new(),
        kind: ToolKind::Execute,
        locations: Vec::new(),
        parent: None,
        started_ms: Some(1),
        status,
        title: "fixture".into(),
    };
    let transcript = Transcript {
        status: Status::Working,
        entries: vec![Entry {
            id: EntryId(1),
            kind: EntryKind::Tool(tool.clone()),
        }],
        activities: activity
            .into_iter()
            .map(|state| Activity {
                call_id: tool.call_id.clone(),
                detail: None,
                finished_ms: None,
                kind: ActivityKind::Shell,
                label: "fixture".into(),
                started_ms: 1,
                state,
            })
            .collect(),
        ..Transcript::new("fixture")
    };
    tool_state(&transcript, &tool)
}
