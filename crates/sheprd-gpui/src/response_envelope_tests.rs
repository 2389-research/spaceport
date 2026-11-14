//! What each reader of a daemon response envelope makes of the same set of
//! envelopes, so the readers cannot drift apart again.
#![allow(clippy::unwrap_used)]

use crate::state::{LiveState, RenameResult};
use serde_json::{Value, json};
use sheprd_client::ClientEvent;

/// The envelopes every reader is pinned against, around a `result` its
/// reader accepts.
pub(crate) fn envelopes(result: Value) -> Vec<(&'static str, Value)> {
    let failed = |error: Value| json!({"id": "r", "error": error});
    let mut wrong = result.clone();
    wrong["type"] = json!("another_method");
    vec![
        ("absent", json!({"id": "r", "result": result})),
        ("null", json!({"id": "r", "error": null, "result": result})),
        ("object", failed(json!({"code": "denied", "message": "no"}))),
        ("no_message", failed(json!({"code": "denied"}))),
        ("no_code", failed(json!({"message": "no"}))),
        ("string", failed(json!("busy"))),
        ("no_result", json!({"id": "r"})),
        ("wrong_type", json!({"id": "r", "result": wrong})),
        (
            "control",
            failed(json!({"code": "denied", "message": "bad\u{1b}[31m\nnews"})),
        ),
        (
            "long",
            failed(json!({"code": "denied", "message": "x".repeat(2000)})),
        ),
    ]
}

/// One reader's outcome for every envelope, with control characters escaped
/// and a long text cut to its length.
pub(crate) fn outcomes(
    result: Value,
    read: impl Fn(&Value) -> String,
) -> Vec<(&'static str, String)> {
    envelopes(result)
        .into_iter()
        .map(|(case, response)| {
            let outcome = read(&response).escape_debug().to_string();
            let length = outcome.chars().count();
            let outcome = if length > 200 {
                let head: String = outcome.chars().take(12).collect();
                format!("{head}..({length})")
            } else {
                outcome
            };
            (case, outcome)
        })
        .collect()
}

/// Compares one reader's outcomes with its pinned table.
pub(crate) fn assert_outcomes(
    reader: &str,
    actual: Vec<(&'static str, String)>,
    pinned: [(&str, &str); 10],
) {
    let actual: Vec<_> = actual
        .iter()
        .map(|(case, outcome)| (*case, outcome.as_str()))
        .collect();
    assert_eq!(actual, pinned, "{reader}");
}

fn renamed(response: &Value, request: &str) -> String {
    let mut state = LiveState::default();
    state.tab_rename = Some(RenameResult {
        request: "rename".into(),
        result: None,
    });
    state.apply(ClientEvent::Response {
        request_id: request.into(),
        response: response.clone(),
    });
    let rename = match state.tab_rename.unwrap().result {
        None => "pending".into(),
        Some(Ok(())) => "ok".into(),
        Some(Err(error)) => format!("err {error}"),
    };
    format!("{rename} | {}", state.error.as_deref().unwrap_or("-"))
}

#[test]
fn window_readers_agree_on_each_envelope() {
    assert_outcomes(
        "tab start",
        outcomes(
            json!({"type": "tab_created", "tab": {"tab_id": "t"}, "root_pane": {"pane_id": "p"}}),
            |response| match crate::window::tab_start::created_tab(response) {
                Ok(Some(_)) => "tab".into(),
                Ok(None) => "none".into(),
                Err(error) => format!("err {error}"),
            },
        ),
        [
            ("absent", "tab"),
            ("null", "tab"),
            ("object", "err no"),
            ("no_message", "err denied"),
            ("no_code", "err no"),
            ("string", "err busy"),
            ("no_result", "none"),
            ("wrong_type", "none"),
            ("control", "err bad[31mnews"),
            ("long", "err xxxxxxxx..(1028)"),
        ],
    );
    assert_outcomes(
        "rename answer",
        outcomes(json!({"type": "ok"}), |response| {
            renamed(response, "rename")
        }),
        [
            ("absent", "ok | -"),
            ("null", "ok | -"),
            ("object", "err no | -"),
            ("no_message", "err denied | -"),
            ("no_code", "err no | -"),
            ("string", "err busy | -"),
            ("no_result", "ok | -"),
            ("wrong_type", "ok | -"),
            ("control", "err bad[31mnews | -"),
            ("long", "err xxxxxxxx..(1032)"),
        ],
    );
    assert_outcomes(
        "unowned answer",
        outcomes(json!({"type": "ok"}), |response| renamed(response, "other")),
        [
            ("absent", "pending | -"),
            ("null", "pending | -"),
            ("object", "pending | no"),
            ("no_message", "pending | denied"),
            ("no_code", "pending | no"),
            ("string", "pending | busy"),
            ("no_result", "pending | -"),
            ("wrong_type", "pending | -"),
            ("control", "pending | bad[31mnews"),
            ("long", "pending | xx..(1034)"),
        ],
    );
}

#[test]
fn workspace_dialogs_agree_on_each_envelope() {
    assert_outcomes(
        "workspace dialog",
        outcomes(json!({"type": "worktree_opened"}), |response| {
            crate::menu::refusal(response).unwrap_or_else(|| "-".into())
        }),
        [
            ("absent", "-"),
            ("null", "-"),
            ("object", "denied: no"),
            ("no_message", "denied: denied"),
            ("no_code", "endpoint_error: no"),
            ("string", "endpoint_error: busy"),
            ("no_result", "-"),
            ("wrong_type", "-"),
            ("control", "denied: bad[31mnews"),
            ("long", "denied: xxxx..(1032)"),
        ],
    );
}
