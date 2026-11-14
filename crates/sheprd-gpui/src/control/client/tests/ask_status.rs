// ABOUTME: Verifies the human status output reports only concrete Ask controller facts.
// ABOUTME: Bounds and sanitizes diagnostic text from a status response.
use crate::control::{client::app::describe_ask, protocol::app::AskStatus};

#[test]
fn ask_status_output_distinguishes_absence_and_actual_controller_facts() {
    assert!(describe_ask(None).contains("controller not initialized"));
    let facts = AskStatus {
        configured: true,
        loading: false,
        answering: true,
        history_present: true,
        agent: Some("Pi".into()),
        error: None,
    };
    let line = describe_ask(Some(&facts));
    for fact in [
        "configured yes",
        "agent Pi",
        "loading no",
        "active answer yes",
        "history present yes",
    ] {
        assert!(line.contains(fact), "{line}");
    }
    assert!(!line.contains("stopped"));
    assert!(!line.contains("unknown"));
}

#[test]
fn ask_status_output_bounds_and_sanitizes_diagnostics() {
    let facts = AskStatus {
        error: Some(format!("\u{1b}\n{}", "🙂".repeat(2_000))),
        ..Default::default()
    };
    let line = describe_ask(Some(&facts));
    assert!(!line.contains('\u{1b}'));
    assert!(line.chars().count() < 700);
    assert!(line.contains("error:"));
}

#[test]
fn status_help_describes_ask_inspection_without_submitting() {
    let text = super::help::usage(Some("status"));
    assert!(text.contains("Ask Spaceport"));
    assert!(text.contains("No question is submitted"));
}
