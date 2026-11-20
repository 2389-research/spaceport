// ABOUTME: Checks legacy tab actions, rename, and move choices survive active menu reset.
// ABOUTME: Opens each page through its real tab target before observing departure.
use super::*;
use crate::sidebar::layout_tests::fixture_window;
use gpui::{Context, Modifiers, TestAppContext};

fn fixture(window: &mut Window, cx: &mut Context<SheprdWindow>) -> SheprdWindow {
    let mut view = fixture_window(window, cx);
    view.live.snapshot = Some(std::sync::Arc::new(
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../sheprd-protocol/tests/fixtures/endpoint-snapshot-v1.json"
        )))
        .expect("synthetic tab snapshot"),
    ));
    view.endpoints[0].connection.target = sheprd_client::ConnectTarget::Local;
    view
}

#[gpui::test]
fn tab_pages_keep_their_body_after_live_target_reset(cx: &mut TestAppContext) {
    let mut geometry = Vec::new();
    for &(page, selector) in &[
        (Page::Tab, "tab-menu-0"),
        (Page::RenameTab, "tab-rename-submit"),
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        (Page::MoveTab, "tab-move-0"),
    ] {
        let (view, cx) = cx.add_window_view(fixture);
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                if page == Page::RenameTab {
                    view.rename_focused_tab(window, cx);
                } else {
                    view.open_tab_menu("w1:t1", None, point(px(500.), px(200.)), window, cx);
                }
            });
            window.refresh();
            window.draw(cx).clear(cx);
        });
        if page == Page::MoveTab {
            let row = cx
                .debug_bounds("tab-menu-2")
                .expect("the move action is offered");
            cx.simulate_click(row.center(), Modifiers::default());
            cx.update(|window, cx| window.draw(cx).clear(cx));
        }
        view.read_with(cx, |view, _| assert_eq!(view.menu.page, Some(page)));
        let before = cx.debug_bounds(selector).expect("the live body is shown");
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                view.menu.appear.live = true;
                view.dismiss_menu(window, cx);
                assert!(view.menu.tab.is_none());
                let later = motion::now() + std::time::Duration::from_secs(3600);
                view.menu.appear.observe(None, None, later);
                let leaving = view.menu.appear.leaving.as_mut().expect("tab page leaves");
                leaving.motion.set(true, later, true);
                leaving.motion.set(false, later, false);
            });
            window.refresh();
            window.draw(cx).clear(cx);
        });
        geometry.push((page, cx.debug_bounds(selector), before));
    }
    let changed: Vec<_> = geometry
        .into_iter()
        .filter(|(_, after, before)| *after != Some(*before))
        .collect();
    assert!(
        changed.is_empty(),
        "departing tab page geometry changed: {changed:?}"
    );
}
