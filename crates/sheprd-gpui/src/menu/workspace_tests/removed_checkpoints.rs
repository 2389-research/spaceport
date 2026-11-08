// ABOUTME: Guards the workspace menu after the checkpoint dialog is retired.
// ABOUTME: Keeps the note action on the same selected checkout.
use super::*;

#[gpui::test]
fn workspace_menu_omits_checkpoints_and_keeps_notes(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.live.local_daemon_peer = true;
            view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
            view.open_workspace_menu("w4", Default::default(), window, cx);
            let items = view.workspace_items();
            assert!(
                items
                    .iter()
                    .all(|(_, label)| !label.english().contains("Checkpoint"))
            );
            assert!(items.iter().any(|(action, _)| {
                *action == WorkspaceMenuAction::Dialog(WorkspaceAction::Note)
            }));
            assert_eq!(
                view.menu.target.as_ref().map(|target| target.id.as_str()),
                Some("w4")
            );
        })
    });
}
