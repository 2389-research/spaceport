#![allow(clippy::unwrap_used)]
use super::*;
use crate::protocol::{Call, Caller, MAX_WAIT_SECONDS, Request, Response};

mod removed_cloud;
mod removed_github;

#[test]
fn host_requests_have_a_stable_wire_form() {
    let request = Request::Host(Call {
        caller: Caller::default(),
        op: HostOp::Forward { port: 3000 },
    });
    let text = serde_json::to_string(&request).unwrap();
    assert_eq!(
        text,
        r#"{"method":"host","caller":{"pane_id":null,"workspace_id":null,"daemon_socket":null},"op":{"op":"forward","port":3000}}"#
    );
    assert_eq!(serde_json::from_str::<Request>(&text).unwrap(), request);
    for op in [
        HostOp::List,
        HostOp::Select {
            id: "devbox".into(),
        },
        HostOp::Ports,
        HostOp::Unforward { port: 1 },
        HostOp::OpenPort { port: 8080 },
        HostOp::Usage,
        HostOp::System,
    ] {
        let request = Request::Host(Call {
            caller: Caller::default(),
            op,
        });
        let text = serde_json::to_string(&request).unwrap();
        assert_eq!(serde_json::from_str::<Request>(&text).unwrap(), request);
    }
}

#[test]
fn unknown_host_operations_and_fields_are_rejected() {
    for invalid in [
        r#"{"method":"host","caller":{},"op":{"op":"destroy"}}"#,
        r#"{"method":"host","caller":{},"op":{"op":"forward"}}"#,
        r#"{"method":"host","caller":{},"op":{"op":"forward","port":70000}}"#,
        r#"{"method":"host","caller":{},"op":{"op":"select","id":"a","all":true}}"#,
    ] {
        assert!(
            serde_json::from_str::<Request>(invalid).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn only_waiting_operations_extend_the_answer_timeout() {
    let wait = |op| {
        Request::Host(Call {
            caller: Caller::default(),
            op,
        })
        .wait_seconds()
    };
    assert_eq!(wait(HostOp::OpenPort { port: 80 }), FORWARD_WAIT_SECONDS);
    assert!(FORWARD_WAIT_SECONDS <= MAX_WAIT_SECONDS);
}

#[test]
fn host_replies_round_trip() {
    for reply in [
        Reply::Ports(Box::new(Ports {
            host: "Local".into(),
            workspace: "w_1".into(),
            listening: vec![PortInfo {
                port: 3000,
                bind: "*:3000".into(),
                process: Some("node".into()),
                local_url: Some("http://localhost:3000/".into()),
            }],
            forwards: vec![ForwardInfo {
                host: "me@devbox".into(),
                port: 5173,
                state: "ended".into(),
                local_url: None,
                reason: Some("ssh exited".into()),
            }],
            note: None,
        })),
        Reply::Usage {
            host: "Local".into(),
            usage: vec![UsageInfo {
                provider: "Claude".into(),
                account: None,
                plan: Some("Max".into()),
                windows: vec![UsageWindow {
                    label: "Session".into(),
                    used_percent: 12.5,
                    resets_in_seconds: Some(60),
                }],
                error: None,
            }],
            note: None,
        },
        Reply::System {
            load: None,
            note: Some("Not sampled".into()),
        },
    ] {
        let response = Response::Host { reply };
        let text = serde_json::to_string(&response).unwrap();
        assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), response);
    }
}
