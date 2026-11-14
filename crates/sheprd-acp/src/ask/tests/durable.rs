// ABOUTME: Verifies bounded atomic durable state and exclusive ownership.
// ABOUTME: Retains the last valid checkpoint when replacement fails.
use super::*;

#[test]
fn checkpoint_roundtrip_and_size_failure_preserve_last_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = checkpoint::Store::open(dir.path()).unwrap();
    let state = checkpoint();
    store.save(&state).unwrap();
    assert_eq!(store.load().unwrap(), Some(state.clone()));
    assert!(checkpoint::Store::open(dir.path()).is_err());
    let mut large = state.clone();
    large.request = Some(Request {
        id: CommandId::random(),
        text: "x".repeat(sheprd_chat_protocol::ask::MAX_CHECKPOINT_BYTES),
        phase: Phase::Intent,
    });
    assert!(store.save(&large).is_err());
    assert_eq!(store.load().unwrap(), Some(state));
}
