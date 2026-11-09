use super::*;

#[gpui::test]
fn every_git_checkout_offers_its_scripts(cx: &mut gpui::TestAppContext) {
    use crate::worktree_scripts::ScriptKind;
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
            // A main checkout runs, a linked one also sets up, a plain folder neither.
            for (id, run, setup) in [
                ("w3", true, false),
                ("w4", true, true),
                ("w0", false, false),
            ] {
                view.open_workspace_menu(id, Default::default(), window, cx);
                let actions = items(view);
                assert_eq!(
                    actions.contains(&WorkspaceMenuAction::Script(ScriptKind::Run)),
                    run,
                    "{id}"
                );
                assert_eq!(
                    actions.contains(&WorkspaceMenuAction::Script(ScriptKind::Setup)),
                    setup,
                    "{id}"
                );
                view.dismiss_menu(window, cx);
            }
        })
    });
}
