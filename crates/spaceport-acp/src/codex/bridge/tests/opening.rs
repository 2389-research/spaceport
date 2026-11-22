// ABOUTME: Exercises Codex opening failures through the translator boundary.
// ABOUTME: Keeps handshake errors observable before and after session requests.
use super::*;

fn refused(id: &Value, message: &str) -> Value {
    json!({"id": id, "error": {"code": -1, "message": message}})
}

#[test]
fn an_early_handshake_error_answers_later_openings() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    assert!(
        bridge
            .on_harness(&refused(&harness(&start[0])["id"], "offline"))
            .is_empty()
    );
    for id in [1, 2] {
        let outs = bridge.on_session(request(id, "session/new", json!({"cwd": "/work"})));
        assert_eq!(
            outs.len(),
            1,
            "the failed handshake must answer the opening"
        );
        assert_eq!(
            response(&outs[0], id).as_ref().unwrap_err().message,
            "Codex's handshake failed: offline"
        );
    }
}

#[test]
fn a_pending_opening_gets_the_handshake_error_once() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    bridge.on_session(request(1, "session/new", json!({})));
    let error = refused(&harness(&start[0])["id"], "offline");
    let outs = bridge.on_harness(&error);
    assert_eq!(
        response(&outs[0], 1).as_ref().unwrap_err().message,
        "Codex's handshake failed: offline"
    );
    assert!(bridge.on_harness(&error).is_empty());
}

#[test]
fn a_missing_thread_id_fails_instead_of_opening() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    bridge.on_harness(&answer(&harness(&start[0])["id"], json!({})));
    let calls = bridge.on_session(request(1, "session/new", json!({})));
    bridge.on_harness(&answer(&harness(&calls[0])["id"], models()));
    let outs = bridge.on_harness(&answer(
        &harness(&calls[1])["id"],
        json!({"thread": {"id": ""}}),
    ));
    assert!(
        response(&outs[0], 1).is_err(),
        "an empty thread id cannot open a session"
    );
}

#[test]
fn a_refused_thread_answers_without_waiting_for_models() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    bridge.on_session(request(1, "session/new", json!({})));
    let calls = bridge.on_harness(&answer(&harness(&start[0])["id"], json!({})));
    let outs = bridge.on_harness(&refused(&harness(&calls[2])["id"], "bad thread"));
    assert_eq!(
        response(&outs[0], 1).as_ref().unwrap_err().message,
        "bad thread"
    );
    assert!(
        bridge
            .on_harness(&answer(&harness(&calls[1])["id"], models()))
            .is_empty()
    );
}

#[test]
fn missing_models_still_opens_a_valid_thread() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    bridge.on_harness(&answer(&harness(&start[0])["id"], json!({})));
    let calls = bridge.on_session(request(1, "session/new", json!({})));
    bridge.on_harness(&refused(&harness(&calls[0])["id"], "no models"));
    let outs = bridge.on_harness(&answer(
        &harness(&calls[1])["id"],
        json!({"thread": {"id": THREAD}}),
    ));
    assert_eq!(response(&outs[0], 1).as_ref().unwrap()["sessionId"], THREAD);
}

#[test]
fn overlapping_opening_does_not_lose_the_first_request() {
    let mut bridge = Bridge::default();
    let start = bridge.start();
    bridge.on_session(request(1, "session/new", json!({})));
    let outs = bridge.on_session(request(2, "session/new", json!({})));
    assert_eq!(outs.len(), 1);
    assert!(response(&outs[0], 2).is_err());
    let outs = bridge.on_harness(&refused(&harness(&start[0])["id"], "offline"));
    assert!(response(&outs[0], 1).is_err());
}

#[test]
fn a_finished_opening_can_be_followed_by_another_session() {
    let mut bridge = opened();
    let outs = bridge.on_session(request(
        2,
        "session/load",
        json!({"sessionId": "next-thread", "cwd": "/work"}),
    ));
    let resume = harness(&outs[0]);
    assert_eq!(resume["method"], "thread/resume");
    let outs = bridge.on_harness(&answer(
        &resume["id"],
        json!({"thread": {"id": "next-thread"}}),
    ));
    assert_eq!(
        response(&outs[0], 2).as_ref().unwrap()["sessionId"],
        "next-thread"
    );
}
