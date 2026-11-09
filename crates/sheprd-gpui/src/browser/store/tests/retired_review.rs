// ABOUTME: Checks old native review tabs retire without losing Core tabs.
// ABOUTME: Reserves discarded IDs so stale group picks never select a new tab.
use super::*;

#[test]
fn old_review_is_dropped_without_reusing_its_id() {
    let scope = Scope::endpoint("local");
    let mut old = Store::default();
    let devices = old
        .open(scope.clone(), "w", Some(Location::Devices), None)
        .unwrap();
    let core = old
        .open(
            scope.clone(),
            "w",
            Some(Location::CoreTerminal {
                panes: crate::core_terminals::panes::Panes::single(
                    sheprd_backend::TerminalId::try_from("t1".to_owned()).unwrap(),
                ),
                observe: false,
            }),
            None,
        )
        .unwrap();
    let review = old.open(scope.clone(), "w", None, None).unwrap();
    let mut saved = serde_json::to_value(Saved {
        tabs: old.tabs,
        next: 0,
    })
    .unwrap();
    let path = std::env::current_dir().unwrap();
    saved["tabs"][2]["location"] = serde_json::json!({
        "kind": "review", "checkout": {"repo_key": path, "branch": "main", "checkout": path}
    });

    let bytes = serde_json::to_vec(&saved).unwrap();
    let mut restored = Store::with_tabs(parse(&bytes).unwrap(), None);
    assert_eq!(
        restored.tabs().map(|tab| tab.id).collect::<Vec<_>>(),
        [devices, core]
    );
    assert!(restored.get(review).is_none());
    assert!(u64::from(restored.open(scope, "w", None, None).unwrap()) > u64::from(review));
}

#[test]
fn old_review_checkout_still_rejects_relative_paths() {
    let bytes = br#"{"tabs":[{"id":0,"scope":"local","workspace_id":"w","location":{"kind":"review","checkout":{"repo_key":"relative/.git","branch":"main","checkout":null}},"title":"Review"}]}"#;
    assert!(parse(bytes).is_err());
}

#[test]
fn retired_review_id_stays_reserved_after_a_second_restore() {
    let scope = Scope::endpoint("local");
    let mut old = Store::default();
    let devices = old
        .open(scope.clone(), "w", Some(Location::Devices), None)
        .unwrap();
    let review = old.open(scope.clone(), "w", None, None).unwrap();
    let mut saved = serde_json::to_value(Saved {
        tabs: old.tabs,
        next: 0,
    })
    .unwrap();
    let path = std::env::current_dir().unwrap();
    saved["tabs"][1]["location"] = serde_json::json!({
        "kind": "review", "checkout": {"repo_key": path, "branch": "main", "checkout": path}
    });

    let first = Store::with_tabs(parse(&serde_json::to_vec(&saved).unwrap()).unwrap(), None);
    assert_eq!(
        first.tabs().map(|tab| tab.id).collect::<Vec<_>>(),
        [devices]
    );
    let persisted = serde_json::to_vec(&first.saved()).unwrap();
    let mut second = Store::with_saved(parse_saved(&persisted).unwrap(), None);
    assert!(second.open(scope, "w", None, None).unwrap() > review);
}
