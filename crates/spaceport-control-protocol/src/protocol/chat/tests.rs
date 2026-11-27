#![allow(clippy::unwrap_used)]
use super::*;
use crate::protocol::{Call, Caller, Request, Response};

#[test]
fn chat_requests_have_a_stable_wire_form() {
    let request = Request::Chat(Call {
        caller: Caller::default(),
        op: ChatOp::View {
            switch: Switch::Off,
            pane: Some("w1:p2".into()),
        },
    });
    let text = serde_json::to_string(&request).unwrap();
    assert_eq!(
        text,
        r#"{"method":"chat","caller":{"pane_id":null,"workspace_id":null,"daemon_socket":null},"op":{"op":"view","switch":"off","pane":"w1:p2"}}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&text).unwrap(), request);
    let status: Request =
        serde_json::from_str(r#"{"method":"chat","caller":{},"op":{"op":"status"}}"#).unwrap();
    assert!(matches!(
        status,
        Request::Chat(Call {
            op: ChatOp::Status,
            ..
        })
    ));
    for invalid in [
        r#"{"method":"chat","caller":{},"op":{"op":"prompt","text":"x"}}"#,
        r#"{"method":"chat","caller":{},"op":{"op":"view","switch":"on","path":"/tmp/x"}}"#,
    ] {
        assert!(
            serde_json::from_str::<Request>(invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn chat_replies_round_trip() {
    for reply in [
        Reply::Chats {
            chats: vec![ChatInfo {
                pane_id: "w1:p1".into(),
                agent: "Claude".into(),
                status: "working".into(),
                attached: true,
                shown: true,
                permissions: 1,
                queued: 0,
                model: None,
                input_mode: InputMode::Chat,
                session_id: None,
                context: None,
            }],
        },
        Reply::View {
            pane_id: "w1:p1".into(),
            shown: false,
        },
        Reply::Created {
            agent: "claude".into(),
            window_id: 7,
            tab: Tab::Core {
                tab_id: 9,
                terminal_id: "terminal-9".into(),
            },
            pane_id: "core:terminal-9".into(),
        },
    ] {
        let response = Response::Chat { reply };
        let text = serde_json::to_string(&response).unwrap();
        assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), response);
    }
}

#[test]
fn chat_new_has_a_stable_wire_form() {
    let request = Request::Chat(Call {
        caller: Caller::default(),
        op: ChatOp::New {
            agent: "claude".into(),
            pane: None,
        },
    });
    let text = serde_json::to_string(&request).unwrap();
    assert!(
        text.ends_with(r#""op":{"op":"new","agent":"claude","pane":null}}"#),
        "{text}"
    );
    assert_eq!(serde_json::from_str::<Request>(&text).unwrap(), request);
    let created = Response::Chat {
        reply: Reply::Created {
            agent: "claude".into(),
            window_id: 7,
            tab: Tab::Herdr {
                tab_id: "w1:t9".into(),
            },
            pane_id: "w1:p9".into(),
        },
    };
    let text = serde_json::to_string(&created).unwrap();
    assert!(
        text.contains(r#""tab":{"runtime":"herdr","tab_id":"w1:t9"}"#),
        "{text}"
    );
    assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), created);
}

#[test]
fn chat_creation_has_a_bounded_acknowledgement_wait() {
    let request = Request::Chat(Call {
        caller: Caller::default(),
        op: ChatOp::New {
            agent: "claude".into(),
            pane: None,
        },
    });
    assert_eq!(request.wait_seconds(), CREATION_WAIT_SECONDS);
    let status = Request::Chat(Call {
        caller: Caller::default(),
        op: ChatOp::Status,
    });
    assert_eq!(status.wait_seconds(), 0);
}
