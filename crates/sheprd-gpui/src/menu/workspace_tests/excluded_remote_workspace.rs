use super::*;

#[gpui::test]
fn legacy_new_workspace_stays_local_and_core_create_opens(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            snapshot.workspaces = sidebar::layout_tests::snapshot(7).workspaces;
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
            let mut remote = crate::endpoint::Endpoint::new(
                "ssh:box".into(),
                "Box".into(),
                sheprd_client::ConnectTarget::Ssh {
                    target: "nobody@box.invalid".into(),
                    session: "default".into(),
                },
                true,
            );
            remote.live = view.live.clone();
            view.endpoints.push(remote);

            view.open_workspace_menu("w3", Default::default(), window, cx);
            view.open_workspace_dialog(WorkspaceAction::NewWorkspace, window, cx);
            assert_eq!(
                view.menu.target.as_ref().map(|target| target.id.as_str()),
                Some("w3")
            );
            assert_eq!(
                view.menu.page,
                Some(super::super::Page::Dialog(WorkspaceAction::NewWorkspace))
            );
        });
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("menu-panel").is_some());
    assert!(cx.debug_bounds("dispatch-picker").is_none());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_core_workspace_dialog(window, cx);
            assert_eq!(
                view.menu.page,
                Some(super::super::Page::CoreWorkspaceCreate)
            );
        })
    });
}
