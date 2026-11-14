// ABOUTME: Checks structured observation output without a model or running daemon.
// ABOUTME: Preserves incomplete coverage and maps unavailable results to honest exits.
use super::*;
use crate::control::protocol::observation::*;
mod coverage;
#[test]
fn unavailable_observations_keep_typed_json_and_nonzero_exit() {
    let result = super::super::observation::output(Reply::Unavailable {
        target: Some(Target("source-1".into())),
        reason: UnavailableReason::StaleTarget,
    })
    .unwrap();
    assert_eq!(result.status, exit::NOT_FOUND);
    assert_eq!(result.json["reason"], "stale_target");
}
#[test]
fn help_names_the_three_observations_and_prepared_scope() {
    let text = help::usage(Some("observation"));
    for word in ["sessions", "conversation", "terminal", "prepared"] {
        assert!(text.contains(word));
    }
}
