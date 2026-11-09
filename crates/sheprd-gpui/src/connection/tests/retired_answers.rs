#![allow(clippy::unwrap_used)]
use super::*;

fn answered(id: &str) -> Option<(String, Option<crate::state::DialogResponse>)> {
    Some((id.into(), Some(Ok(serde_json::json!({"result": {}})))))
}

/// An answer read while the event reader held the mailbox is cleared at
/// the next update instead of being copied into every one after it.
#[test]
fn an_answer_read_during_contention_is_cleared_at_the_next_update() {
    let bridge = bridge();
    {
        let mut inbox = bridge.inbox.lock().unwrap();
        inbox.script_response = answered("list");
        inbox.dirty = true;
    }
    let held = bridge.inbox.lock().unwrap();
    bridge.retire_answer(Answer::Script, "list");
    drop(held);
    // A newer script request took the slot before the update.
    bridge.inbox.lock().unwrap().script_response = Some(("tab".into(), None));

    let update = bridge.take_update().unwrap();
    assert!(matches!(&update.script_response, Some((id, None)) if id == "tab"));
    assert!(bridge.retired.lock().unwrap().is_empty());
}
