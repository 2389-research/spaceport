//! The exact prompt reply, permission request and settings the bridge
//! sends the session: the contract its shared builders must keep.
use super::*;

#[test]
fn a_finished_turn_sends_exactly_this_reply() {
    let mut bridge = opened();
    prompt(&mut bridge, 2, "hi");
    let outs = bridge.on_harness(&result("success", Some("end_turn")));
    let answer = response(&outs[1], 2).as_ref().unwrap();
    assert_eq!(
        answer,
        &json!({"stopReason": "end_turn", "usage": {
            "inputTokens": 106,
            "outputTokens": 40,
            "cachedReadTokens": 2000,
            "thoughtTokens": 12,
            "totalTokens": 2146,
        }})
    );
    let tokens = crate::project::tokens(answer).unwrap();
    assert_eq!(
        (
            tokens.input,
            tokens.cached,
            tokens.output,
            tokens.thought,
            tokens.total
        ),
        (106, 2000, 40, 12, 2146)
    );
}

#[test]
fn a_result_without_usage_sends_no_usage() {
    let mut bridge = opened();
    prompt(&mut bridge, 2, "hi");
    let mut message = result("success", Some("end_turn"));
    message.as_object_mut().unwrap().remove("usage");
    let outs = bridge.on_harness(&message);
    let answer = response(outs.last().unwrap(), 2).as_ref().unwrap();
    assert_eq!(answer, &json!({"stopReason": "end_turn"}));
    assert!(crate::project::tokens(answer).is_none());
}

#[test]
fn a_failed_turn_and_a_second_prompt_are_refused_with_these_errors() {
    let mut bridge = opened();
    prompt(&mut bridge, 2, "hi");
    let busy = prompt(&mut bridge, 3, "again");
    let error = response(&busy[0], 3).as_ref().unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        json!({"code": -32603, "message": "A turn is already running."})
    );
    let outs = bridge.on_harness(&result("error_during_execution", None));
    let error = response(outs.last().unwrap(), 2).as_ref().unwrap_err();
    assert_eq!(
        serde_json::to_value(error).unwrap(),
        json!({"code": -32603, "message": "Claude stopped: error_during_execution."})
    );
}

#[test]
fn a_permission_request_is_sent_exactly_like_this() {
    let mut bridge = opened();
    prompt(&mut bridge, 2, "run it");
    let outs = bridge.on_harness(&json!({
        "type": "control_request",
        "request_id": "ctl_1",
        "request": {
            "subtype": "can_use_tool",
            "tool_name": "Bash",
            "input": {"command": "ls"},
            "permission_suggestions": [],
            "tool_use_id": "toolu_1",
        },
    }));
    let [
        Out::Session(Inbound::Request {
            id: wire::RequestId::Number(1),
            method,
            params,
        }),
    ] = &outs[..]
    else {
        panic!("expected one request, got {outs:?}");
    };
    assert_eq!(method, "session/request_permission");
    assert_eq!(
        params,
        &json!({
            "sessionId": SESSION,
            "toolCall": {"toolCallId": "toolu_1", "title": "`ls`", "rawInput": {"command": "ls"}},
            "options": [
                {"optionId": "allow", "name": "Yes", "kind": "allow_once"},
                {"optionId": "reject", "name": "No", "kind": "reject_once"},
            ],
        })
    );
}

#[test]
fn the_settings_are_listed_exactly_like_this() {
    let mut bridge = Bridge::new(SESSION.to_owned());
    let start = bridge.start().remove(0);
    let id = claude(&start)["request_id"].as_str().unwrap().to_owned();
    bridge.on_harness(&handshake(&id));
    let outs = bridge.on_session(request(
        1,
        "session/new",
        json!({"cwd": "/work", "mcpServers": []}),
    ));
    let opened = response(&outs[0], 1).as_ref().unwrap();
    assert_eq!(
        opened["configOptions"],
        json!([
            {
                "id": "model", "name": "Model", "category": "model", "type": "select",
                "currentValue": "default",
                "options": [
                    {"value": "default", "name": "Opus 5.5 (recommended)", "description": "Opus 5.5"},
                    {"value": "haiku", "name": "Haiku", "description": "Haiku 5.5"},
                ],
            },
            {
                "id": "effort", "name": "Effort", "category": "thought_level", "type": "select",
                "currentValue": "default",
                "options": [
                    {"value": "default", "name": "Auto"},
                    {"value": "low", "name": "Low"},
                    {"value": "medium", "name": "Medium"},
                    {"value": "high", "name": "High"},
                ],
            },
        ])
    );
}
