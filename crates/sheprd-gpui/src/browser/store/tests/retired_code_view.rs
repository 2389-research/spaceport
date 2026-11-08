// ABOUTME: Checks old source-view tabs retire without losing other saved tabs.
// ABOUTME: Keeps discarded IDs reserved so stale group picks cannot select a new tab.
use super::*;

#[test]
fn old_source_view_is_dropped_without_reusing_its_id() {
    let scope = Scope::endpoint("local");
    let mut old = Store::default();
    let page = old
        .open(scope.clone(), "w", url("https://example.test/"), None)
        .unwrap();
    let px0 = old
        .open_editor_tab(scope.clone(), "w", Editor::Px0, None)
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
    let mut saved = serde_json::to_value(Saved { tabs: old.tabs }).unwrap();
    let path = std::env::current_dir().unwrap().join("main.rs");
    saved["tabs"][3]["location"] = serde_json::json!({
        "kind": "code", "file": {"path": path, "line": 42}
    });

    let bytes = serde_json::to_vec(&saved).unwrap();
    let mut restored = Store::with_tabs(parse(&bytes).unwrap(), None);
    assert_eq!(
        restored.tabs().map(|tab| tab.id).collect::<Vec<_>>(),
        [page, px0, core]
    );
    assert!(restored.get(source).is_none());
    assert_eq!(restored.px0_tab(&scope, "w").map(|tab| tab.id), Some(px0));
    assert!(u64::from(restored.open(scope, "w", None, None).unwrap()) > u64::from(source));
}
