use super::*;

#[gpui::test]
fn reset_drops_target_draft_composition_and_error(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let snapshot = sidebar::layout_tests::snapshot(7);
        let mut menu = super::super::MenuState::new(cx);
        menu.target = Some(WorkspaceTarget::new(&snapshot, &snapshot.workspaces[3]));
        menu.page = Some(super::super::Page::Dialog(WorkspaceAction::Rename));
        let mut input = DialogInput::new("draft".into());
        input.replace(None, "composition", true, None);
        menu.input = Some(input);
        menu.error = Some("old connection error".into());
        menu.reset();
        assert!(menu.page.is_none());
        assert!(menu.target.is_none());
        assert!(menu.input.is_none());
        assert!(menu.error.is_none());
    });
}

#[test]
fn actions_target_clicked_workspace_and_match_daemon_schemas() {
    let mut snapshot = sidebar::layout_tests::snapshot(7);
    snapshot.workspaces[0].branch = None;
    let target = WorkspaceTarget::new(&snapshot, &snapshot.workspaces[3]);
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::Rename, "new label")
            .unwrap(),
        (
            Method::WorkspaceRename,
            serde_json::json!({"workspace_id": "w3", "label": "new label"})
        )
    );
    assert_eq!(
        target
            .request(&snapshot, WorkspaceAction::NewTab, "")
            .unwrap(),
        (
            Method::TabCreate,
            serde_json::json!({"workspace_id": "w3", "focus": true})
        )
    );
    // A stale snapshot is refused before any request names its workspace.
    let mut replaced = snapshot.clone();
    replaced.boot_id = "replacement-boot".into();
    assert!(matches!(
        target.request(&replaced, WorkspaceAction::Rename, "new label"),
        Err(crate::Error::StaleWorkspace)
    ));
}
