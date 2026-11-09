use super::*;

#[gpui::test]
fn local_group_menu_has_no_issues_or_dispatch_action(cx: &mut TestAppContext) {
    let (view, cx) = window(cx);
    let group = groups(&view, cx)[0];
    cx.update(|_, cx| {
        view.update(cx, |view, _| {
            view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
        })
    });
    assert!(
        actions(&view, cx, group)
            .iter()
            .all(|action| action.label() != "Issues and pull requests")
    );
}
