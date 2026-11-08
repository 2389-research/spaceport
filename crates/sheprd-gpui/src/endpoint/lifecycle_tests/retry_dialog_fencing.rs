use super::*;
use crate::menu::Page;

/// Exercise the real retry path against the fixture's missing socket, without
/// discovering a personal daemon or waiting for the backoff clock.
fn retry(view: &mut SheprdWindow, cx: &mut Context<SheprdWindow>) {
    wait_until(|| {
        view.endpoints[0].poll(Instant::now());
        view.endpoints[0].connection.handle.is_none()
    });
    let generation = view.endpoints[0].generation;
    view.endpoints[0].retry_at = Instant::now();
    view.poll_endpoints(cx);
    assert!(view.endpoints[0].generation > generation);
    assert_eq!(view.selected_generation, view.endpoints[0].generation);
}

#[gpui::test]
fn reconnect_and_background_retry_close_daemon_bound_dialogs(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    view.update_in(cx, |view, window, cx| {
        view.open_menu(window, cx);
        view.menu.page = Some(Page::Workspace);
        view.reconnect(cx);
        assert!(view.menu.page.is_none());
        view.open_menu(window, cx);
        view.menu.page = Some(Page::Workspace);
        assert!(view.menu_target_current());
        retry(view, cx);
        assert!(
            view.menu.page.is_none(),
            "daemon-bound dialogs must still close"
        );
        assert!(!view.menu_target_current());
    });
}
