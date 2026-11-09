use super::*;
use crate::{endpoint::Endpoint, state::ConnectionStatus};
use sheprd_client::ConnectTarget;

#[gpui::test]
fn move_picker_keeps_local_rows_without_cross_host_carry(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        crate::bind_keys(cx);
        let mut view = crate::sidebar::layout_tests::fixture_window(window, cx);
        view.endpoints[0].connection.target = ConnectTarget::Local;
        view.live.snapshot = Some(Arc::new(snapshot()));
        let mut remote = Endpoint::new(
            "ssh:there".into(),
            "There".into(),
            ConnectTarget::Ssh {
                target: "nobody@there.invalid".into(),
                session: "default".into(),
            },
            true,
        );
        remote.live.snapshot = Some(Arc::new(snapshot()));
        remote.live.status = ConnectionStatus::Connected;
        view.endpoints.push(remote);
        view
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_pane_menu("inactive", Point::default(), window, cx);
            view.activate_pane_menu(Action::MoveTo, window, cx);
            let pane = view.menu.pane.as_ref().unwrap();
            assert_eq!(view.menu.page, Some(Page::MovePane));
            assert_eq!(
                pane.rows(Some(Page::MovePane)),
                pane.target.destinations().len()
            );
        });
        window.draw(cx).clear(cx);
    });
    assert!(cx.debug_bounds("pane-move-0").is_some());
    assert!(cx.debug_bounds("pane-move-1").is_some());
    assert!(cx.debug_bounds("pane-move-2").is_none());
}
