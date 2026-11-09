// ABOUTME: Checks restoration of saved tabs after VS Code editor tabs retire.
// ABOUTME: Keeps other tab IDs and reserves IDs from discarded entries.
use super::*;

#[test]
fn old_vs_code_tabs_are_dropped_without_reusing_their_ids() {
    let mut old = Store::default();
    let scope = Scope::endpoint("local");
    let devices = old
        .open(scope.clone(), "w1", Some(Location::Devices), None)
        .unwrap();
    let core = old
        .open(
            scope.clone(),
            "w1",
            Some(Location::CoreTerminal {
                panes: crate::core_terminals::panes::Panes::single(
                    sheprd_backend::TerminalId::try_from("t1".to_owned()).unwrap(),
                ),
                observe: false,
            }),
            None,
        )
        .unwrap();
    let code = old.open(scope.clone(), "w1", None, None).unwrap();
    let code_group = old.open(scope.clone(), "w2", None, None).unwrap();
    for tab in &mut old.tabs {
        if tab.id == code || tab.id == code_group {
            tab.place = Place::RetiredVsCode;
        }
    }

    let mut saved = serde_json::to_value(Saved {
        tabs: old.tabs.clone(),
        next: 0,
    })
    .unwrap();
    for tab in saved["tabs"].as_array_mut().unwrap() {
        if tab["id"] == u64::from(code_group) {
            tab["place"] = "code_group".into();
        }
    }
    let bytes = serde_json::to_vec(&saved).unwrap();
    let mut restored = Store::with_tabs(parse(&bytes).unwrap(), None);
    assert_eq!(
        restored.tabs().map(|tab| tab.id).collect::<Vec<_>>(),
        [devices, core]
    );
    assert!(restored.get(code).is_none());
    assert!(restored.get(code_group).is_none());

    let next = restored.open(scope, "w1", None, None).unwrap();
    assert!(u64::from(next) > u64::from(code_group));
}

#[test]
fn discarded_max_id_cannot_wrap_to_a_new_tab() {
    let scope = Scope::endpoint("local");
    let mut old = Store::default();
    old.open(scope.clone(), "w1", None, None).unwrap();
    old.tabs[0].id = TabId::test(u64::MAX);
    old.tabs[0].place = Place::RetiredVsCode;
    let bytes = serde_json::to_vec(&Saved {
        tabs: old.tabs,
        next: 0,
    })
    .unwrap();
    let mut restored = Store::with_tabs(parse(&bytes).unwrap(), None);
    assert!(restored.tabs().next().is_none());
    assert!(restored.open(scope, "w1", None, None).is_none());
}
