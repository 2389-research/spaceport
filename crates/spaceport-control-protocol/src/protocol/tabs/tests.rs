#![allow(clippy::unwrap_used)]
use super::*;
use crate::protocol::{Call, Caller, Request, Response};

mod removed_browser;
mod removed_code_edit;
mod removed_orchestrator;
mod removed_px0;
mod removed_review;

fn round_trip(op: TabOp) {
    let request = Request::Tab(Call {
        caller: Caller::default(),
        op,
    });
    let text = serde_json::to_string(&request).unwrap();
    assert_eq!(serde_json::from_str::<Request>(&text).unwrap(), request);
}

#[test]
fn tab_operations_round_trip() {
    for op in [
        TabOp::List {},
        TabOp::Focus { id: 3 },
        TabOp::Close { id: 3 },
        TabOp::Devices,
    ] {
        round_trip(op);
    }
}

#[test]
fn tab_requests_have_a_stable_wire_form() {
    let request: Request = serde_json::from_str(
        r#"{"method":"tab","caller":{"workspace_id":"w_1"},"op":{"op":"devices"}}"#,
    )
    .unwrap();
    let Request::Tab(call) = request else {
        panic!("not a tab request");
    };
    assert_eq!(call.op, TabOp::Devices);
    assert_eq!(
        serde_json::from_str::<TabOp>(r#"{"op":"list"}"#).unwrap(),
        TabOp::List {}
    );
    for invalid in [
        r#"{"op":"eval","id":1}"#,
        r#"{"op":"text","id":1,"script":"alert(1)"}"#,
        r#"{"op":"focus"}"#,
        r#"{"op":"focus","id":-1}"#,
    ] {
        assert!(serde_json::from_str::<TabOp>(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn removed_code_open_wire_operation_is_rejected() {
    assert!(serde_json::from_str::<TabOp>(r#"{"op":"code","path":"/a.rs"}"#).is_err());
}

#[test]
fn devices_arrival_extends_the_answer_timeout() {
    let wait = |op: TabOp| {
        Request::Tab(Call {
            caller: Caller::default(),
            op,
        })
        .wait_seconds()
    };
    assert_eq!(wait(TabOp::Devices), ARRIVE_WAIT_SECONDS);
    assert_eq!(wait(TabOp::Focus { id: 1 }), 0);
}

#[test]
fn replies_round_trip() {
    let tab = TabInfo {
        id: 7,
        kind: TabKind::CoreTerminal,
        title: "Terminal".into(),
        location: Some("terminal-7".into()),
        workspace: "w_1".into(),
        host: "local:/tmp/herdr.sock".into(),
        opened_by: None,
    };
    for reply in [
        Reply::Tabs { tabs: vec![tab] },
        Reply::Opened {
            id: 7,
            workspace: "w_1".into(),
        },
    ] {
        let response = Response::Tab { reply };
        let text = serde_json::to_string(&response).unwrap();
        assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), response);
    }
    assert_eq!(
        serde_json::to_string(&Reply::Opened {
            id: 7,
            workspace: "w_1".into()
        })
        .unwrap(),
        r#"{"kind":"opened","id":7,"workspace":"w_1"}"#
    );
}

#[test]
fn kinds_are_named_as_json_spells_them() {
    for retired in ["blank", "web", "file"] {
        assert!(serde_json::from_value::<TabKind>(serde_json::json!(retired)).is_err());
    }
    assert!(serde_json::from_str::<TabKind>("\"code\"").is_err());
    assert!(serde_json::from_str::<TabKind>("\"review\"").is_err());
    assert!(serde_json::from_str::<TabKind>("\"editor\"").is_err());
    assert!(serde_json::from_str::<TabKind>("\"orchestrator\"").is_err());
    for kind in [TabKind::CoreTerminal, TabKind::Devices] {
        assert_eq!(
            serde_json::to_string(&kind).unwrap(),
            format!("\"{}\"", kind.name())
        );
    }
}
