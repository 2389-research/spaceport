// ABOUTME: Verifies global ownership and truthful Ask recovery without running an agent.
// ABOUTME: Checks native reading projection and single-turn request boundaries.
use super::{Checkpoint, Outcome, Status, Transcript, controller};
use gpui::TestAppContext;
use sheprd_chat_protocol::{Entry, EntryId, EntryKind};
use uuid::Uuid;
mod status;

fn checkpoint(outcome: Outcome) -> Box<Checkpoint> {
    Box::new(Checkpoint {
        version: sheprd_chat_protocol::ask::VERSION,
        conversation_id: Uuid::new_v4(),
        helper_run_id: Uuid::new_v4(),
        endpoint: sheprd_chat_protocol::EndpointId::random().to_string(),
        pi_session: None,
        pi_session_file: None,
        request: None,
        transcript: Transcript {
            status: Status::Idle,
            entries: vec![Entry {
                id: EntryId(1),
                kind: EntryKind::Agent {
                    text: "Saved answer".into(),
                },
            }],
            ..Transcript::new("Pi")
        },
        outcome,
    })
}

#[gpui::test]
fn opening_windows_shares_one_idle_controller(cx: &mut TestAppContext) {
    cx.update(|cx| {
        super::Feature::install(true, cx);
        let first = controller(cx);
        let second = controller(cx);
        assert_eq!(first.entity_id(), second.entity_id());
        let state = first.read(cx);
        assert!(state.worker.is_none());
        assert!(state.link.is_none());
        assert!(state.pending.is_none());
        assert!(!state.active);
    });
}

#[gpui::test]
fn completed_stopped_and_unknown_history_never_launch_or_resend(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let owner = controller(cx);
        for outcome in [Outcome::Completed, Outcome::Stopped, Outcome::Unknown] {
            owner.update(cx, |owner, cx| owner.restore(Some(checkpoint(outcome)), cx));
            let state = owner.read(cx);
            assert_eq!(state.session.transcript().entries.len(), 1);
            assert!(state.link.is_none());
            assert!(state.pending.is_none());
            assert!(!state.active);
            assert_eq!(state.problem.is_some(), outcome == Outcome::Unknown);
        }
    });
}

#[gpui::test]
fn busy_submit_keeps_the_existing_request_and_never_queues_another(cx: &mut TestAppContext) {
    cx.update(|cx| {
        controller(cx).update(cx, |owner, cx| {
            owner.configured = true;
            owner.active = true;
            owner.pending = Some("Original".into());
            assert!(!owner.submit("Second".into(), cx));
            assert_eq!(owner.pending.as_deref(), Some("Original"));
        });
    });
}

#[gpui::test]
fn unavailable_helper_preserves_an_unsent_draft(cx: &mut TestAppContext) {
    cx.update(|cx| {
        controller(cx).update(cx, |owner, cx| {
            owner.configured = true;
            assert!(!owner.submit("Keep this question".into(), cx));
            assert_eq!(owner.failed_draft.as_deref(), Some("Keep this question"));
            assert!(owner.link.is_none());
            assert!(!owner.active);
        });
    });
}

#[test]
fn reading_projection_copies_visible_markdown_and_unicode() {
    use crate::chat::{prepared::Prepared, selection::Selection};
    let transcript = Transcript {
        entries: vec![Entry {
            id: EntryId(1),
            kind: EntryKind::Agent {
                text: "**Résumé** needs review.\n\n- Check 🛸 status".into(),
            },
        }],
        ..Transcript::new("Pi")
    };
    let mut prepared = Prepared::default();
    prepared.update_reading_lines(&transcript, &[(EntryId(1), 1)]);
    let mut selection = Selection::default();
    assert!(selection.select_all(prepared.text()));
    assert_eq!(
        selection.text(prepared.text()).as_deref(),
        Some("Résumé needs review.\n\nCheck 🛸 status")
    );
}

#[gpui::test]
fn queue_failure_preserves_the_original_unsent_draft(cx: &mut TestAppContext) {
    cx.update(|cx| {
        controller(cx).update(cx, |owner, _| {
            owner.pending = Some("Keep 🛸 this exact draft".into());
            owner.active = true;
            owner.dispatched(Err(crate::chat::Error::NotAttached));
            assert_eq!(
                owner.failed_draft.as_deref(),
                Some("Keep 🛸 this exact draft")
            );
            assert!(!owner.active);
        });
    });
}

#[gpui::test]
fn pre_receipt_failure_preserves_the_submitted_draft_without_replay(cx: &mut TestAppContext) {
    cx.update(|cx| {
        controller(cx).update(cx, |owner, _| {
            owner.awaiting_ack = Some((
                sheprd_chat_protocol::CommandId::random(),
                "Awaiting receipt".into(),
            ));
            owner.active = true;
            owner.fail(crate::chat::Error::Closed.into());
            assert_eq!(owner.failed_draft.as_deref(), Some("Awaiting receipt"));
            assert!(owner.pending.is_none());
            assert!(owner.awaiting_ack.is_none());
        })
    });
}
