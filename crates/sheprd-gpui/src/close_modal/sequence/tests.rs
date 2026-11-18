// ABOUTME: Checks the close deadline survives a removed tab with an unsettled surface.
// ABOUTME: Uses an exact clock and real queued close without sleeps or retrying the mutation.
#![allow(clippy::unwrap_used)]
use super::*;
use gpui::{AppContext as _, Entity, IntoElement, Render};
use std::sync::Arc;

struct Fixture(Entity<SheprdWindow>);
impl Render for Fixture {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::div()
    }
}

#[gpui::test]
fn a_missing_surface_barrier_times_out_after_the_closed_tab_disappears(
    cx: &mut gpui::TestAppContext,
) {
    let mut peer = crate::window::MockPeer::advertising(&["tab.close"]);
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            peer.prepare_local_tab(view, cx);
            let shown = Arc::make_mut(view.live.snapshot.as_mut().unwrap());
            for id in ["first", "second"] {
                let mut tab = shown.tabs[0].clone();
                tab.tab_id = id.into();
                shown.tabs.push(tab);
            }
            *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
            view.close_tabs(
                vec![Pick::Herdr("first".into()), Pick::Herdr("second".into())],
                window,
                cx,
            );
            view.send_close(window, cx);
            assert!(view.browser.closing.is_some());
        })
    });
    assert_eq!(peer.request()["params"]["tab_id"], "first");
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let now = Instant::now();
            Arc::make_mut(view.live.snapshot.as_mut().unwrap())
                .tabs
                .retain(|tab| tab.tab_id != "first");
            view.live.surface = None;
            *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
            view.advance_close_run_at(window, cx, now);
            assert!(view.browser.closing.is_some());
            view.advance_close_run_at(window, cx, now + ANSWER_TIMEOUT + Duration::from_secs(1));
            assert!(
                view.browser.closing.is_none(),
                "a missing barrier must not park the accepted batch forever"
            );
            assert!(view.notices.text().is_some());
        })
    });
}
