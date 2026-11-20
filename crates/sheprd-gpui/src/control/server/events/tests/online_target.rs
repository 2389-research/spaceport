//! `wait online` waits on the selected host of one window, so a caller in a
//! session no window shows is refused at once instead of waiting out the
//! timeout on another session's window.
use super::*;

#[gpui::test]
fn a_wait_for_a_host_of_another_session_is_refused_at_once(cx: &mut gpui::TestAppContext) {
    let (_view, cx) = window(cx);
    let refused = cx
        .cx
        .update(|cx| online_window(&caller(Some("w_other_session")), cx));
    // The window shows w0 and w3, never w_other_session: a pane of a session
    // the app does not show gets exit 5, as every other command does, rather
    // than "the wait was not met" after the timeout.
    assert_eq!(
        error_code(&refused.unwrap_err()),
        Some(ErrorCode::WorkspaceNotFound)
    );
    cx.cx.update(|cx| {
        // The caller's own workspace, or none at all, resolves to a window.
        assert!(online_window(&caller(Some("w3")), cx).is_ok());
        assert!(online_window(&caller(None), cx).is_ok());
    });
}
