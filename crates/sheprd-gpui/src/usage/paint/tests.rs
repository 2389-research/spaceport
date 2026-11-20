// ABOUTME: Checks that usage departure captures concrete data without opaque resource owners.
// ABOUTME: Verifies both supported detail shapes survive independently of the original report.
#![allow(clippy::unwrap_used)]
use super::*;
use std::sync::Arc;

fn report() -> Report {
    Report::new(
        sheprd_usage::registry::find("claude").unwrap(),
        Default::default(),
        Vec::new(),
    )
}

#[test]
fn opaque_details_release_with_the_original_observation() {
    let owner = Arc::new(());
    let weak = Arc::downgrade(&owner);
    let mut report = report();
    report.detail = Some(Arc::new(owner));
    let captured = ReportPaint::from(&report);
    drop(report);
    assert!(weak.upgrade().is_none());
    assert!(captured.standard.detail.is_none());
}

#[test]
fn supported_details_copy_only_their_concrete_values() {
    let mut report = report();
    report.detail = Some(Arc::new(OpenaiDaily {
        days: vec![("fixture day".into(), 1.25)],
    }));
    let daily = ReportPaint::from(&report);
    report.detail = Some(Arc::new(vec![ClaudeAccount {
        email: "fixture@example.invalid".into(),
        alias: Some("Fixture".into()),
        disabled: false,
        status: sheprd_usage::details::ClaudeAccountStatus::Ok,
        stale: false,
        windows: Vec::new(),
    }]));
    let accounts = ReportPaint::from(&report);
    drop(report);
    assert_eq!(
        daily.daily.as_ref().map(|daily| daily.days.as_slice()),
        Some(&[("fixture day".into(), 1.25)][..])
    );
    assert_eq!(
        accounts
            .accounts
            .as_ref()
            .and_then(|accounts| accounts.first())
            .map(ClaudeAccount::name),
        Some("Fixture")
    );
    assert!(daily.accounts.is_none());
    assert!(accounts.daily.is_none());
}
