// ABOUTME: Exercises the Herdr TUI workspace-close action through its rendered menu.
// ABOUTME: Requires explicit confirmation before any workspace processes can end.
use super::*;

#[gpui::test]
fn herdr_tui_workspace_menu_offers_a_close_confirmation(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(sidebar::layout_tests::fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.live.status = crate::state::ConnectionStatus::Connected;
            view.open_workspace_menu("w3", Default::default(), window, cx);
        });
        sidebar::layout_tests::full_draw(window, cx).clear(cx);
    });
    let close = cx.debug_bounds("workspace-menu-Close workspace");
    assert!(close.is_some(), "Herdr TUI workspace closing is missing");
    cx.simulate_click(close.unwrap().center(), Default::default());
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.menu.page, Some(super::super::Page::ConfirmClose));
        assert!(view.menu.close.is_some());
    });
}
