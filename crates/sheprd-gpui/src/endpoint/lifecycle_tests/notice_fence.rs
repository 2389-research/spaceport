// ABOUTME: Pins which connection changes a click on a toast or system notification survives.
// ABOUTME: A notice reaches the connection it came from, selected or not, while that connection exists.

use super::*;
use crate::endpoint::fence::{
    ConnectionRef,
    changes::{CLICK_CHANGES, CLICK_REACHES, apply_click},
};

/// A window whose selected remote endpoint shows one navigable toast.
fn noticed(
    view: &gpui::Entity<SheprdWindow>,
    cx: &mut gpui::VisualTestContext,
) -> (Server, u64, Arc<Mutex<LiveState>>) {
    let (mut endpoint, server) = connected_endpoint("ssh:toast");
    let mut wire = crate::notifications::tests::notification("fenced");
    wire.workspace_id = Some("w1".into());
    wire.pane_id = Some("w1:p1".into());
    endpoint
        .toasts
        .receive([crate::notifications::Notice::new(wire, Instant::now())
            .with_snapshot(endpoint.live.snapshot.as_deref())
            .preview()]);
    let inbox = endpoint.connection.inbox.clone();
    let generation = view.update(cx, |view, cx| {
        view.endpoints.truncate(1);
        view.endpoints[0].detached = true;
        view.endpoints.push(endpoint);
        view.selected_endpoint = 1;
        view.options = ConnectOptions::default();
        view.reset_selected(cx);
        view.tick_toasts(false, Instant::now(), cx);
        view.endpoints[1].generation
    });
    (server, generation, inbox)
}

#[gpui::test]
fn a_toast_click_reaches_its_own_connection_while_it_exists(cx: &mut gpui::TestAppContext) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let seen = CLICK_CHANGES.map(|change| {
        let (_server, generation, inbox) = noticed(&view, cx);
        view.update(cx, |view, cx| {
            apply_click(view, 1, change, cx);
            // A held inbox keeps an accepted click pending, where it shows.
            let _held = inbox.lock().unwrap();
            let source = ConnectionRef::for_test("ssh:toast", generation, &inbox);
            view.click_toast(&source, 0, cx);
            view.pending_toast == Some(0)
        })
    });
    assert_eq!(seen, CLICK_REACHES);
}

#[gpui::test]
fn a_system_notification_click_reaches_its_own_connection_while_it_exists(
    cx: &mut gpui::TestAppContext,
) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let seen = CLICK_CHANGES.map(|change| {
        let (_server, generation, inbox) = noticed(&view, cx);
        let posted = ConnectionRef::for_test("ssh:toast", generation, &inbox).downgrade();
        cx.update(|window, cx| {
            view.update(cx, |view, cx| {
                apply_click(view, 1, change, cx);
                let _held = inbox.lock().unwrap();
                view.open_system_notification(&posted, 0, window, cx);
                view.pending_toast == Some(0)
            })
        })
    });
    assert_eq!(seen, CLICK_REACHES);
}

/// The registry holds the inbox weakly: once the connection's state is
/// dropped, the click reaches nothing, whatever the endpoint looks like.
#[gpui::test]
fn a_system_notification_click_reaches_nothing_once_its_inbox_is_dropped(
    cx: &mut gpui::TestAppContext,
) {
    let (fixture, cx) = cx.add_window_view(|window, cx| {
        Fixture(cx.new(|cx| crate::sidebar::layout_tests::fixture_window(window, cx)))
    });
    let view = fixture.update(cx, |fixture, _| fixture.0.clone());
    let (_server, generation, inbox) = noticed(&view, cx);
    // The posting connection's state is gone; the endpoint itself is as it was.
    let gone = Arc::new(Mutex::new(LiveState::default()));
    let posted = ConnectionRef::for_test("ssh:toast", generation, &gone).downgrade();
    drop(gone);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            let _held = inbox.lock().unwrap();
            view.open_system_notification(&posted, 0, window, cx);
            assert!(view.pending_toast.is_none());
        })
    });
}
