// ABOUTME: Confirms exact Core split-pane closure without any Herdr TUI endpoint.
// ABOUTME: Keeps stale selection refusal while exercising actual terminal.kill frames.
use super::*;

#[gpui::test]
fn a_core_only_split_close_uses_its_core_target(cx: &mut TestAppContext) {
    let (handle, tab) = split_window(cx);
    let mut peer = connected(&handle, cx);
    handle
        .update(cx, |view, window, cx| {
            view.endpoints.clear();
            view.live.snapshot = None;
            view.command(Command::ClosePane, window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
            assert!(!view.menu_target_current());
            view.send_close(window, cx);
            assert!(
                view.menu.page.is_none(),
                "valid Core consent must not require a legacy endpoint"
            );
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1", "t3"]);
    receive_kill(&mut peer, "t2");
}

#[gpui::test]
fn a_core_only_close_still_refuses_a_changed_selection_epoch(cx: &mut TestAppContext) {
    let (handle, tab) = split_window(cx);
    let _peer = connected(&handle, cx);
    handle
        .update(cx, |view, window, cx| {
            view.endpoints.clear();
            view.live.snapshot = None;
            view.command(Command::ClosePane, window, cx);
            view.selection_epoch += 1;
            view.send_close(window, cx);
            assert_eq!(view.menu.page, Some(Page::ConfirmClose));
        })
        .unwrap();
    assert_eq!(saved_terminals(tab, cx), ["t1", "t2", "t3"]);
}
