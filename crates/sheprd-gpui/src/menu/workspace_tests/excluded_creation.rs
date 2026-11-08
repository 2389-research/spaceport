use super::*;

#[gpui::test]
fn workspace_menu_has_no_worktree_creation_but_keeps_ordinary_actions(
    cx: &mut gpui::TestAppContext,
) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.open_workspace_menu("w3", Default::default(), window, cx);
        })
    });
    let actions = cx.update(|_, cx| view.read(cx).workspace_items());
    assert!(
        !actions
            .iter()
            .any(|(_, label)| { label.english().contains("New worktree") })
    );
    assert!(
        actions
            .iter()
            .any(|(action, _)| { *action == WorkspaceMenuAction::Dialog(WorkspaceAction::Rename) })
    );
}
