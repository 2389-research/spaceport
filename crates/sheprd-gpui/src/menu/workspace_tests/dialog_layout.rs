use super::*;

#[gpui::test]
fn workspace_dialog_centres_on_window(cx: &mut gpui::TestAppContext) {
    use gpui::{point, px};
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.simulate_resize(gpui::size(px(800.), px(600.)));
    let centre = point(px(400.), px(300.));
    for anchor in [point(px(120.), px(140.)), point(px(700.), px(520.))] {
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
                snapshot.workspaces = sidebar::layout_tests::snapshot(7).workspaces;
                view.live.status = crate::state::ConnectionStatus::Connected;
                view.menu.reset();
                view.open_workspace_menu("w4", anchor, window, cx);
            });
            window.draw(cx).clear(cx);
        });
        let menu = cx.debug_bounds("menu-panel").unwrap();
        assert!(
            (menu.center() - centre).x.abs() > px(40.)
                || (menu.center() - centre).y.abs() > px(40.)
        );
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.open_workspace_dialog(WorkspaceAction::DeleteWorktree, window, cx)
            });
            window.draw(cx).clear(cx);
        });
        let panel = cx.debug_bounds("menu-panel").unwrap();
        let offset = panel.center() - centre;
        assert!(offset.x.abs() <= px(1.) && offset.y.abs() <= px(1.));
        assert!(panel.size.width <= px(480.) && panel.size.width >= px(400.));
    }
}

#[gpui::test]
fn delete_dialog_keeps_path_and_buttons_inside_panel(cx: &mut gpui::TestAppContext) {
    use gpui::px;
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    for (width, height) in [(640., 400.), (1200., 780.)] {
        cx.simulate_resize(gpui::size(px(width), px(height)));
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                let snapshot = std::sync::Arc::make_mut(view.live.snapshot.as_mut().unwrap());
                snapshot.workspaces = sidebar::layout_tests::snapshot(7).workspaces;
                view.live.status = crate::state::ConnectionStatus::Connected;
                view.menu.reset();
                view.open_workspace_menu("w4", Default::default(), window, cx);
                view.open_workspace_dialog(WorkspaceAction::DeleteWorktree, window, cx);
                view.menu.deletion = Some(Deletion {
                    path: Some("/endpoint/.herdr/worktrees/agent-launcher/child".into()),
                    archive: ArchiveCheck::Read(None),
                    ..Deletion::new(None, true)
                });
                view.menu.error = Some("fixture error".into());
            });
            window.draw(cx).clear(cx);
        });
        let panel = cx.debug_bounds("menu-panel").unwrap();
        let cancel = cx.debug_bounds("dialog-cancel").unwrap();
        let submit = cx.debug_bounds("dialog-submit").unwrap();
        let error = cx.debug_bounds("dialog-error").unwrap();
        let path = cx.debug_bounds("dialog-path").unwrap();
        assert!(cx.update(|_, cx| view.read(cx).menu.input.is_none()));
        for part in [cancel, submit, error, path] {
            assert!(part.left() >= panel.left() && part.right() <= panel.right());
            assert!(part.right() <= px(width));
            if height > 400. {
                assert!(part.top() >= panel.top() && part.bottom() <= panel.bottom());
            }
        }
        assert!(cancel.right() <= submit.left());
        assert!(submit.right() < panel.right());
        assert!(path.bottom() <= error.top());
        assert!(error.bottom() <= cancel.top());
    }
}
