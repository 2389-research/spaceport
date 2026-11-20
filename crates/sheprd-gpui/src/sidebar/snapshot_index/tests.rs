// ABOUTME: Verifies exact snapshot identity, source release, and borrowed row ownership.
// ABOUTME: Keeps transport clones and unchanged projections compatible with cached metadata.
#![allow(clippy::unwrap_used)]
use super::*;

fn snapshot() -> ClientShellSnapshot {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
    )))
    .unwrap()
}

fn with_snapshot(snapshot: Arc<ClientShellSnapshot>) -> LiveState {
    let mut live = LiveState::default();
    live.snapshot = Some(snapshot);
    live
}

#[test]
fn unchanged_projections_reuse_the_index_without_retaining_source_records() {
    let mut live = with_snapshot(Arc::new(snapshot()));
    let source = live.snapshot.as_ref().unwrap();
    let weak = Arc::downgrade(source);
    let index = live.sidebar_index.for_snapshot(source);
    assert_eq!(Arc::strong_count(source), 1);
    live.apply_window_update(with_snapshot(live.snapshot.as_ref().unwrap().clone()));
    assert!(Arc::ptr_eq(
        &index,
        &live
            .sidebar_index
            .for_snapshot(live.snapshot.as_ref().unwrap())
    ));
    let indexed = live.indexed_snapshot().unwrap();
    assert!(std::ptr::eq(
        indexed.pane("w1:p1").unwrap(),
        &indexed.snapshot.panes[0]
    ));
    live.snapshot = None;
    assert!(
        weak.upgrade().is_none(),
        "index retained its source snapshot"
    );
    fn assert_send<T: Send>() {}
    assert_send::<LiveState>();
}

#[test]
fn make_mut_invalidates_identity_without_a_boot_or_revision_change() {
    let mut live = with_snapshot(Arc::new(snapshot()));
    let first = live
        .sidebar_index
        .for_snapshot(live.snapshot.as_ref().unwrap());
    let boot = live.snapshot.as_ref().unwrap().boot_id.clone();
    let revision = live.snapshot.as_ref().unwrap().revision;
    Arc::make_mut(live.snapshot.as_mut().unwrap()).panes[0].pane_id = "replacement".into();
    let next = live
        .sidebar_index
        .for_snapshot(live.snapshot.as_ref().unwrap());
    assert!(!Arc::ptr_eq(&first, &next));
    let rows = live.indexed_snapshot().unwrap();
    assert_eq!(rows.snapshot.boot_id, boot);
    assert_eq!(rows.snapshot.revision, revision);
    assert!(rows.pane("w1:p1").is_none());
    assert_eq!(rows.pane("replacement").unwrap().pane_id, "replacement");
}

#[test]
fn new_allocations_rebuild_same_revision_plugin_order() {
    let mut source = snapshot();
    source.agent_view_label = Some("filtered".into());
    source.agent_order = vec!["w1:p1".into()];
    let mut live = with_snapshot(Arc::new(source));
    assert_eq!(
        live.indexed_snapshot()
            .unwrap()
            .agents(AgentSort::Grouped)
            .len(),
        1
    );
    let mut next = live.snapshot.as_ref().unwrap().as_ref().clone();
    next.agent_order.clear();
    live.apply_window_update(with_snapshot(Arc::new(next)));
    assert!(
        live.indexed_snapshot()
            .unwrap()
            .agents(AgentSort::Grouped)
            .is_empty()
    );
}

#[test]
fn launch_roots_keep_the_first_minimum_tab_and_first_matching_pane() {
    let mut source = snapshot();
    let mut tab = source.tabs[0].clone();
    tab.tab_id = "later".into();
    source.tabs.push(tab);
    let mut pane = source.panes[0].clone();
    pane.tab_id = "later".into();
    pane.cwd = Some("/later".into());
    source.panes.push(pane);
    assert_eq!(
        IndexedSnapshot::new(&source).launch_root("w1"),
        Some("/repo")
    );
    source.panes[0].cwd = None;
    assert_eq!(IndexedSnapshot::new(&source).launch_root("w1"), None);
}
