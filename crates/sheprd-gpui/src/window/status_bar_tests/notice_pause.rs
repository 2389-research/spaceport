// ABOUTME: Checks warning reading time across actual selected-inbox projection updates.
// ABOUTME: Prevents stale hover pauses from surviving newer warnings or connection resets.
use super::*;
use crate::{notice::Cause, state::LiveState, window::MockPeer};
use gpui::Context;
use std::time::{Duration, Instant};

fn publish(view: &mut SheprdWindow, mut state: LiveState, cx: &mut Context<SheprdWindow>) {
    state.dirty = true;
    *view.endpoints[0].connection.inbox.lock().unwrap() = state;
    view.poll_endpoints(cx);
}

#[gpui::test]
fn selected_worker_updates_preserve_the_same_warnings_reading_time(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    let peer = MockPeer::advertising(&[]);
    view.update(cx, |view, cx| {
        peer.prepare_local_tab(view, cx);
        view.status_bar_visible = false;
        let start = Instant::now();
        view.live
            .notices
            .show(Cause::Daemon, "Request refused".into());
        let worker = view.live.clone();
        view.live
            .notices
            .pause(true, start + Duration::from_secs(2));
        publish(view, worker.clone(), cx);
        view.expire_status_notices(start + Duration::from_secs(100));
        assert_eq!(view.live.notices.text(), Some("Request refused"));

        view.live
            .notices
            .pause(false, start + Duration::from_secs(102));
        publish(view, worker, cx);
        view.expire_status_notices(start + Duration::from_secs(109));
        assert_eq!(view.live.notices.text(), Some("Request refused"));
        view.expire_status_notices(start + Duration::from_secs(111));
        assert!(view.live.notices.is_none());
    });
}

#[gpui::test]
fn newer_warnings_and_explicit_context_reset_do_not_inherit_an_old_pause(cx: &mut TestAppContext) {
    for reset in [false, true] {
        let (view, cx) = cx.add_window_view(fixture_window);
        let peer = MockPeer::advertising(&[]);
        view.update(cx, |view, cx| {
            peer.prepare_local_tab(view, cx);
            view.status_bar_visible = false;
            view.live
                .notices
                .show(Cause::Daemon, "First refusal".into());
            let mut worker = view.live.clone();
            let start = Instant::now();
            view.live.notices.pause(true, start);
            if !reset {
                worker.notices.show(Cause::Daemon, "New refusal".into());
            }
            publish(view, worker, cx);
            if reset {
                view.reset_selected(cx);
            }
            view.expire_status_notices(start + Duration::from_secs(11));
            assert!(view.live.notices.is_none());
        });
    }
}
