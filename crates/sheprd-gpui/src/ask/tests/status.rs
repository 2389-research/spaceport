// ABOUTME: Verifies bounded public Ask inspection reads the actual controller.
// ABOUTME: Ensures status inspection never starts a helper or submits a question.
use super::super::controller;
use gpui::TestAppContext;

#[gpui::test]
fn public_status_reads_bounded_controller_facts_without_starting_a_helper(cx: &mut TestAppContext) {
    cx.update(|cx| {
        assert!(super::super::status(cx).is_none());
        super::super::Feature::install(true, cx);
        let owner = controller(cx);
        owner.update(cx, |owner, _| {
            owner.configured = true;
            owner.loading = true;
            owner.active = true;
            owner.problem = Some(format!("\u{1b}{}", "🙂".repeat(2_000)));
        });
        let facts = super::super::status(cx).unwrap_or_default();
        assert!(facts.configured && facts.loading && facts.answering);
        assert!(!facts.history_present);
        assert_eq!(facts.agent.as_deref(), Some("Pi"));
        assert!(
            facts
                .error
                .is_some_and(|error| error.chars().count() <= 512 && !error.contains('\u{1b}'))
        );
        let owner = owner.read(cx);
        assert!(owner.link.is_none() && owner.worker.is_none() && owner.pending.is_none());
    });
}
