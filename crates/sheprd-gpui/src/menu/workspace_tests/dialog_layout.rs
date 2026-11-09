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
                view.open_workspace_dialog(WorkspaceAction::Rename, window, cx)
            });
            window.draw(cx).clear(cx);
        });
        let panel = cx.debug_bounds("menu-panel").unwrap();
        let offset = panel.center() - centre;
        assert!(offset.x.abs() <= px(1.) && offset.y.abs() <= px(1.));
        assert!(panel.size.width <= px(420.) && panel.size.width >= px(400.));
    }
}
