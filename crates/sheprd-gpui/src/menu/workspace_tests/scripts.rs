use super::*;

#[gpui::test]
fn workspace_menu_keeps_checkout_actions_without_script_launches(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = crate::state::ConnectionStatus::Connected;
            let items = |view: &SheprdWindow| -> Vec<_> {
                view.workspace_items()
                    .into_iter()
                    .map(|(action, _)| action)
                    .collect()
            };
            for id in ["w3", "w4", "w0"] {
                view.open_workspace_menu(id, Default::default(), window, cx);
                let actions = items(view);
                assert!(
                    actions.iter().all(|action| matches!(
                        action,
                        WorkspaceMenuAction::Dialog(_)
                            | WorkspaceMenuAction::Collapse
                            | WorkspaceMenuAction::Expand
                            | WorkspaceMenuAction::Mute
                            | WorkspaceMenuAction::Unmute
                    )),
                    "{id}: {actions:?}"
                );
                assert!(actions.contains(&WorkspaceMenuAction::Dialog(WorkspaceAction::Rename)));
                view.dismiss_menu(window, cx);
            }
        })
    });
}
