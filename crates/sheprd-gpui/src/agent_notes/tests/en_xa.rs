//! The `en-XA` leak test for the reasons feedback was kept.
use super::*;

#[test]
fn each_reason_notes_were_kept_has_its_own_sentence() {
    let kept = [
        Kept::NoAgent,
        Kept::NotReady,
        Kept::PaneGone,
        Kept::Unreachable,
    ]
    .map(|reason| crate::i18n::fl!("agent-notes-kept", reason = reason.selector()));
    assert_eq!(
        kept,
        [
            "No agent runs in that pane; notes kept for `browser feedback`",
            "The agent is not ready; notes kept for `browser feedback`",
            "The agent's pane is not here; notes kept for `browser feedback`",
            "Could not reach the agent; notes kept for `browser feedback`",
        ]
    );
}
