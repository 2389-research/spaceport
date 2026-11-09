use super::*;

#[gpui::test]
fn local_workspace_menu_has_no_issues_or_dispatch_action(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
            view.open_workspace_menu("w3", Default::default(), window, cx);
        })
    });
    let items = cx.update(|_, cx| view.read(cx).workspace_items());
    assert!(
        items
            .iter()
            .all(|(_, label)| label.english() != "Issues and pull requests")
    );
}
