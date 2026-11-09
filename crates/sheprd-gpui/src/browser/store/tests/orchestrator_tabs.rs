use super::*;

#[test]
fn orchestrator_tabs_saved_by_earlier_builds_are_dropped_on_load() {
    let mut store = Store::default();
    let scope = Scope::endpoint("local");
    let devices = store
        .open(scope.clone(), "w_1", Some(Location::Devices), None)
        .unwrap();
    let orchestrator = store
        .open(
            scope,
            "w_1",
            Some(Location::RetiredOrchestrator {
                repo: crate::browser::RetiredOrchestratorRepo::new("/tmp/repo".into()).unwrap(),
            }),
            None,
        )
        .unwrap();
    let bytes = serde_json::to_vec(&Saved {
        tabs: store.tabs.clone(),
        next: 0,
    })
    .unwrap();
    // The file still reads, so retained views come back.
    let restored = Store::with_tabs(parse(&bytes).unwrap(), None);
    assert!(restored.get(devices).is_some());
    assert!(restored.get(orchestrator).is_none());
}
