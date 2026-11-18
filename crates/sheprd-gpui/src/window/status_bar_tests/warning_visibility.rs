// ABOUTME: Exercises operation warnings in the actual hidden-status-bar layout.
// ABOUTME: Preserves the authoritative disconnection card and private window state.
use super::*;
use crate::{notice::Cause, state::ConnectionStatus};

#[gpui::test]
fn operation_failures_remain_visible_without_the_status_bar(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        view.live.status = ConnectionStatus::Connected;
        view.status_bar_visible = false;
        view.notices
            .show(Cause::Command, "Could not open that tab".into());
        cx.notify();
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("operation-warning").is_some());
    assert!(cx.debug_bounds("connection-status").is_none());
}

#[gpui::test]
fn operation_and_gesture_warnings_remain_visible_in_zen_mode(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, cx| {
        view.zen.mode = super::super::zen::Zen::On {
            entered_fullscreen: false,
            restore_font: None,
        };
        view.live.status = ConnectionStatus::Connected;
        view.notices
            .show(Cause::Command, "Could not open that tab".into());
        view.show_flash(crate::window::Flash::warning("Could not copy"), cx);
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("operation-warning").is_some());
    assert!(cx.debug_bounds("flash").is_some());
    assert!(cx.debug_bounds("connection-status").is_none());
}

#[gpui::test]
fn a_local_failure_stays_visible_beside_a_connection_failure(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update(cx, |view, _| {
        view.live.status = ConnectionStatus::Disconnected;
        view.live
            .notices
            .show(Cause::Connection, "Connection closed".into());
        view.notices
            .show(Cause::Command, "Could not open that tab".into());
    });
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("connection-status").is_some());
    view.read_with(cx, |view, _| {
        let status = view.live.status_text(view.notices.text());
        assert!(status.contains("Connection closed"));
        assert!(!status.contains("Could not open that tab"));
    });
    assert!(cx.debug_bounds("operation-warning").is_some());
}

#[gpui::test]
fn opening_history_resumes_a_hovered_warnings_deadline(cx: &mut TestAppContext) {
    for local in [false, true] {
        let (view, cx) = cx.add_window_view(fixture_window);
        view.update(cx, |view, _| {
            view.status_bar_visible = false;
            view.live.status = ConnectionStatus::Connected;
            let notices = if local {
                &mut view.notices
            } else {
                &mut view.live.notices
            };
            notices.show(Cause::Command, "Could not open that tab".into());
        });
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let card = cx.debug_bounds("operation-warning").unwrap();
        cx.simulate_mouse_move(card.center(), None, gpui::Modifiers::default());
        view.read_with(cx, |view, _| {
            let notices = if local {
                &view.notices
            } else {
                &view.live.notices
            };
            assert!(notices.is_paused(), "the visible owner pauses");
        });
        cx.update(|window, cx| view.update(cx, |view, cx| view.open_messages(window, cx)));
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        assert!(cx.debug_bounds("operation-warning").is_none());
        view.update(cx, |view, _| {
            assert!(!view.notices.is_paused());
            assert!(!view.live.notices.is_paused());
            view.expire_status_notices(
                std::time::Instant::now() + std::time::Duration::from_secs(100),
            );
            assert!(view.notices.is_none() && view.live.notices.is_none());
        });
    }
}
