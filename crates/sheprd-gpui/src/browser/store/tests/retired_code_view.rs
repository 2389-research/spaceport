// ABOUTME: Checks old source-view tabs retire without losing other saved tabs.
// ABOUTME: Keeps discarded IDs reserved so stale group picks cannot select a new tab.
use super::*;

#[test]
fn old_source_view_is_dropped_without_reusing_its_id() {
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
    let source = old.open(scope.clone(), "w", None, None).unwrap();
    let mut saved = serde_json::to_value(Saved {
        tabs: old.tabs,
        next: 0,
    })
    .unwrap();
    let path = std::env::current_dir().unwrap().join("main.rs");
    saved["tabs"][2]["location"] = serde_json::json!({
        "kind": "code", "file": {"path": path, "line": 42}
    });

    let bytes = serde_json::to_vec(&saved).unwrap();
    let mut restored = Store::restored_for_test(&bytes).unwrap();
    assert_eq!(
        restored.tabs().map(|tab| tab.id).collect::<Vec<_>>(),
        [devices, core]
    );
    assert!(restored.get(source).is_none());
    assert!(u64::from(restored.open(scope, "w", None, None).unwrap()) > u64::from(source));
}
