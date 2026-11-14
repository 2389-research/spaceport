// ABOUTME: Tests durable request reconciliation around dispatch and cancellation.
// ABOUTME: Prevents queue receipts or process exit from proving acceptance.
use super::*;

#[test]
fn unobserved_dispatch_stays_unknown_on_reopen() {
    let mut state = checkpoint();
    state.request = Some(Request {
        id: CommandId::random(),
        text: "question".into(),
        phase: Phase::Intent,
    });
    state.outcome = Outcome::Answering;
    reconcile(&mut state);
    assert_eq!(state.request.as_ref().unwrap().phase, Phase::Unknown);
    assert_eq!(state.outcome, Outcome::Unknown);
    assert!(continuation(&state).is_none());
}

#[test]
fn accepted_interruption_resumes_but_explicit_stop_never_does() {
    let mut state = checkpoint();
    state.request = Some(Request {
        id: CommandId::random(),
        text: "question".into(),
        phase: Phase::Accepted,
    });
    state.outcome = Outcome::Answering;
    finish(&mut state, TurnEnd::Cancelled, true);
    assert_eq!(state.outcome, Outcome::Interrupted);
    state.pi_session = Some("exact-pi-id".into());
    state.pi_session_file = Some("/owned/sessions/pi.jsonl".into());
    assert!(continuation(&state).is_some());
    state.request.as_mut().unwrap().phase = Phase::Stopped;
    state.outcome = Outcome::Stopped;
    finish(&mut state, TurnEnd::Cancelled, true);
    assert_eq!(state.outcome, Outcome::Stopped);
    assert!(continuation(&state).is_none());
}

#[test]
fn cancellation_without_acceptance_does_not_make_a_resume_receipt() {
    let mut state = checkpoint();
    state.request = Some(Request {
        id: CommandId::random(),
        text: "question".into(),
        phase: Phase::Intent,
    });
    state.outcome = Outcome::Answering;
    finish(&mut state, TurnEnd::Cancelled, true);
    assert_eq!(state.outcome, Outcome::Unknown);
}
