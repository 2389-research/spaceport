// ABOUTME: Verifies pane popup bodies remain after their active owners reset.
// ABOUTME: Covers action, rename, move, process list, and end-process confirmation pages.
use super::*;
use gpui::TestAppContext;

#[gpui::test]
fn pane_pages_keep_their_body_after_live_target_reset(cx: &mut TestAppContext) {
    let mut changed = Vec::new();
    for &(page, selector) in &[
        (Page::Pane, "pane-menu-0"),
        (Page::RenamePane, "pane-rename-submit"),
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        (Page::MovePane, "pane-move-0"),
        (Page::PaneProcesses, "pane-process-1"),
        (Page::KillProcesses, "pane-processes-confirm"),
    ] {
        let (view, cx) =
            cx.add_window_view(|window, cx| crate::pane_menu::departure_fixture(page, window, cx));
        cx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(view.read_with(cx, |view, _| view.menu.page), Some(page));
        let before = cx
            .debug_bounds(selector)
            .expect("the live pane body is shown");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.menu.appear.live = true;
                view.dismiss_menu(window, cx);
                assert!(view.menu.pane.is_none());
                let later = motion::now() + std::time::Duration::from_secs(3600);
                view.menu.appear.observe(None, None, later);
                let leaving = view.menu.appear.leaving.as_mut().expect("pane page leaves");
                leaving.motion.set(true, later, true);
                leaving.motion.set(false, later, false);
            });
            window.refresh();
            window.draw(cx).clear(cx);
        });
        let after = cx.debug_bounds(selector);
        if after != Some(before) {
            changed.push((page, before, after));
        }
    }
    assert!(
        changed.is_empty(),
        "departing pane geometry changed: {changed:?}"
    );
}
