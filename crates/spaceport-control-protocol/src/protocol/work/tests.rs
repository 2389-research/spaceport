#![allow(clippy::unwrap_used)]
use super::*;
use crate::protocol::{Call, Caller, Request, Response};

mod excluded_fan_out;
mod removed_checkpoint;
mod removed_teleport;

#[test]
fn work_requests_have_a_stable_wire_form() {
    for op in [
        WorkOp::Notes,
        WorkOp::Note,
        WorkOp::SetNote { text: "x".into() },
        WorkOp::MuteStatus,
    ] {
        let request = Request::Work(Call {
            caller: Caller::default(),
            op,
        });
        let text = serde_json::to_string(&request).unwrap();
        assert_eq!(serde_json::from_str::<Request>(&text).unwrap(), request);
    }
}

#[test]
fn malformed_work_requests_are_rejected() {
    for invalid in [
        r#"{"method":"work","caller":{},"op":{"op":"restore"}}"#,
        r#"{"method":"work","caller":{},"op":{"op":"script","kind":"deploy"}}"#,
        r#"{"method":"work","caller":{},"op":{"op":"script","kind":"archive"}}"#,
        r#"{"method":"work","caller":{},"op":{"op":"script","kind":"setup"}}"#,
        r#"{"method":"work","caller":{},"op":{"op":"script","kind":"run"}}"#,
        r#"{"method":"work","caller":{},"op":{"op":"set_note"}}"#,
        r#"{"method":"work","caller":{},"op":{"op":"fan_out","prompt":"x","agents":[["claude",300]]}}"#,
        r#"{"method":"work","caller":{},"op":{"op":"launch"}}"#,
    ] {
        assert!(
            serde_json::from_str::<Request>(invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn work_replies_round_trip() {
    let note = NoteInfo {
        host: "local".into(),
        repository: "/src/app/.git".into(),
        branch: "main".into(),
        text: "waiting on review".into(),
    };
    for response in [
        Response::Work {
            reply: Reply::Notes {
                notes: vec![note.clone()],
                omitted: 2,
            },
        },
        Response::Work {
            reply: Reply::Note { note: Some(note) },
        },
        Response::Work {
            reply: Reply::Note { note: None },
        },
    ] {
        let text = serde_json::to_string(&response).unwrap();
        assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), response);
    }
}

#[test]
fn mute_requests_keep_their_wire_names() {
    let op: WorkOp = serde_json::from_str(r#"{"op":"mute","muted":true}"#).unwrap();
    assert_eq!(op, WorkOp::Mute { muted: true });
    let op: WorkOp = serde_json::from_str(r#"{"op":"mute_status"}"#).unwrap();
    assert_eq!(op, WorkOp::MuteStatus);
}
