// ABOUTME: Keeps contextual group rows visible after their active menu resets.
// ABOUTME: Exercises departure through the real window and prepared group geometry.
use super::*;
use crate::sidebar::layout_tests::fixture_window;
use gpui::TestAppContext;

#[gpui::test]
fn group_rows_leave_after_the_live_menu_drops(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        let snapshot =
            std::sync::Arc::make_mut(view.live.snapshot.as_mut().expect("fixture snapshot"));
        snapshot.focused_workspace_id = Some("w0".into());
        snapshot.focused_tab_id = Some("t0".into());
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let group = view.group_slots()[0].id;
            view.open_group_menu(group, point(px(500.), px(200.)), window, cx);
        });
        window.draw(cx).clear(cx);
    });
    let before = cx
        .debug_bounds("group-menu-Split")
        .expect("the live row appears");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
            assert!(view.menu.group.is_none());
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view
                .menu
                .appear
                .leaving
                .as_mut()
                .expect("group menu leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        });
        window.draw(cx).clear(cx);
    });
    assert_eq!(
        cx.debug_bounds("group-menu-Split"),
        Some(before),
        "the departing menu must retain its visible row geometry"
    );
}
