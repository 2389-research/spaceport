// ABOUTME: Verifies a new tab starts only from the workspace and tab shown to the user.
// ABOUTME: Keeps input paused until Herdr identifies the created tab and confirms its surface.
use super::*;
use serde_json::json;

fn new_tab() -> Navigation {
    Navigation::NewTab {
        workspace: "w1".into(),
        source_tab: "w1:t1".into(),
    }
}

#[test]
fn creation_rejects_a_changed_workspace_or_source_tab() {
    let mut snapshot = snapshot();
    assert!(new_tab().present(&snapshot));
    snapshot.focused_tab_id = Some("w1:t2".into());
    assert!(!new_tab().present(&snapshot));
    snapshot.focused_tab_id = Some("w1:t1".into());
    snapshot.focused_workspace_id = Some("w2".into());
    assert!(!new_tab().present(&snapshot));
}

#[test]
fn only_the_created_tab_unlocks_the_new_terminal() {
    let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
    state.snapshot(Arc::new(snapshot()));
    state.surface(Arc::new(surface()));
    state.start_fence("create".into(), new_tab(), "barrier".into());
    state.receipt(
        "create",
        &json!({"result":{"type":"tab_created","tab":{"tab_id":"w1:t2"}}}),
    );
    state.receipt("barrier", &json!({"result":{"type":"client_shell_surface_set","active":true,"projection_revision":8}}));
    let mut next = snapshot();
    next.revision = 8;
    let mut frame = surface();
    frame.projection_revision = 8;
    state.snapshot(Arc::new(next.clone()));
    state.surface(Arc::new(frame.clone()));
    assert!(state.target().is_none());
    next.focused_tab_id = Some("w1:t2".into());
    next.tabs[0].tab_id = "w1:t2".into();
    next.panes[0].tab_id = "w1:t2".into();
    next.panes[0].pane_id = "w1:p2".into();
    next.focused_pane_id = Some("w1:p2".into());
    frame.panes[0].pane_id = "w1:p2".into();
    state.snapshot(Arc::new(next));
    state.surface(Arc::new(frame));
    assert_eq!(state.target().unwrap().pane_id, "w1:p2");
}

#[test]
fn an_unidentified_created_tab_leaves_input_closed() {
    for receipt in [
        json!({"id": "create"}),
        json!({"id": "create", "result": null}),
        json!({"result":{"type":"tab_created","tab":{"tab_id":""}}}),
        json!({"result":{"type":"tab_created","tab":{"tab_id":"w1:t1"}}}),
    ] {
        let mut state = Projection::new(geometry(3, 2, 8, 16).unwrap());
        state.snapshot(Arc::new(snapshot()));
        state.start_fence("create".into(), new_tab(), "barrier".into());
        state.receipt("create", &receipt);
        assert!(state.disconnected, "{receipt}");
        assert!(state.target().is_none());
    }
}
