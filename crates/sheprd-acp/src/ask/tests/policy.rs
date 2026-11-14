// ABOUTME: Proves Ask accepts only literal idle prompts and Stop.
// ABOUTME: Prevents permission, settings, attachment, and queue execution.
use super::*;

#[test]
fn accepts_only_plain_idle_questions_and_stop() {
    let mut state = checkpoint();
    state.transcript.status = Status::Idle;
    let prompt = Command::Prompt {
        text: "/bash literal question".into(),
        attachments: vec![],
    };
    assert!(allowed(&state, &prompt));
    assert!(!allowed(&state, &Command::SetAutoAccept { on: true }));
    state.outcome = Outcome::Answering;
    assert!(!allowed(&state, &prompt));
    assert!(allowed(&state, &Command::Cancel));
}

#[test]
fn no_uncertain_request_is_automatically_continued() {
    let mut state = checkpoint();
    state.pi_session = Some("exact-pi-id".into());
    state.pi_session_file = Some("/owned/session.jsonl".into());
    state.request = Some(Request {
        id: CommandId::random(),
        text: "question".into(),
        phase: Phase::Unknown,
    });
    assert!(continuation(&state).is_none());
    state.request.as_mut().unwrap().phase = Phase::Accepted;
    assert!(continuation(&state).is_none());
    state.request.as_mut().unwrap().phase = Phase::Interrupted;
    state.outcome = Outcome::Interrupted;
    assert!(continuation(&state).unwrap().contains("question"));
    state.outcome = Outcome::Stopped;
    assert!(continuation(&state).is_none());
}
