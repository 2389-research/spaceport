//! A workspace drag's life after the press: the click it stays when it never
//! lifts, the drop that moves nothing, and the settle between a sent move and
//! the daemon's order, which a stray release or Escape must not end.

use super::*;
use crate::{
    reorder::{LIFT_DELAY, SETTLE_TIMEOUT},
    sidebar::layout_tests::{fixture_window, full_draw, snapshot},
    state::ConnectionStatus,
    window::MockPeer,
};
use gpui::{
    Bounds, Entity, Modifiers, MouseButton, TestAppContext, VisualTestContext, point, px, size,
};
use std::sync::Arc;

const FIRST: &str = "row-herdr";
const SECOND: &str = "row-herdr-gpui-sidebar-rendering-regression-investigation";

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
}

/// A connected window whose moves reach `peer`, drawn once.
fn connected<'a>(
    peer: &MockPeer,
    cx: &'a mut TestAppContext,
) -> (Entity<SheprdWindow>, &'a mut VisualTestContext) {
    let handle = peer.client.handle.clone();
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.status = ConnectionStatus::Connected;
        view.endpoints[0].connection.handle = Some(handle);
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    draw(cx);
    (view, cx)
}

/// Whether a drag is held, floats, and previews its drop in the list.
fn phase(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) -> (bool, bool, bool) {
    view.read_with(cx, |view, _| {
        let workspaces = &view.live.snapshot.as_ref().unwrap().workspaces;
        view.workspace_drag
            .as_ref()
            .map_or((false, false, false), |drag| {
                (true, drag.floating(), drag.previewing(workspaces))
            })
    })
}

/// Lifts the first row and drops it before the third, so the move is sent and
/// the list previews it. Returns the first two rows' resting bounds.
fn drop_first_before_third(
    view: &Entity<SheprdWindow>,
    cx: &mut VisualTestContext,
) -> (Bounds<Pixels>, Bounds<Pixels>) {
    let first = cx.debug_bounds(FIRST).unwrap();
    let second = cx.debug_bounds(SECOND).unwrap();
    cx.simulate_mouse_down(first.center(), MouseButton::Left, Modifiers::default());
    cx.executor().advance_clock(LIFT_DELAY);
    cx.run_until_parked();
    let below = point(first.center().x, second.bottom() - px(2.));
    for _ in 0..2 {
        cx.simulate_mouse_move(below, MouseButton::Left, Modifiers::default());
        draw(cx);
    }
    cx.simulate_mouse_up(below, MouseButton::Left, Modifiers::default());
    draw(cx);
    assert_eq!(
        phase(view, cx),
        (true, false, true),
        "the drop settles with its preview"
    );
    assert_eq!(view.read_with(cx, |view, _| view.local_error.clone()), None);
    (first, second)
}

/// The daemon's answer to the drop: the first workspace now sits third.
fn daemon_reorders(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    view.update(cx, |view, cx| {
        let mut reordered = snapshot(40);
        reordered.workspaces[..2].rotate_left(1);
        view.live.snapshot = Some(Arc::new(reordered));
        cx.notify();
    });
}

/// A press released before it lifts is a click: the release reports that it
/// is not the drag's, and nothing is left armed for the lift timer.
#[gpui::test]
fn a_release_before_the_lift_stays_a_click(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    let first = cx.debug_bounds(FIRST).unwrap();
    cx.simulate_mouse_down(first.center(), MouseButton::Left, Modifiers::default());
    assert_eq!(phase(&view, cx), (true, false, false));
    let consumed = view.update(cx, |view, cx| view.release_workspace_drag(cx));
    assert!(!consumed, "an unlifted release is left to the click");
    cx.executor().advance_clock(LIFT_DELAY * 2);
    cx.run_until_parked();
    assert_eq!(phase(&view, cx), (false, false, false));
}

/// A lifted row released over its own place moves nothing and settles
/// nothing, but the release is still the drag's, not a click.
#[gpui::test]
fn a_lifted_row_released_in_place_ends_the_drag(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    let first = cx.debug_bounds(FIRST).unwrap();
    cx.simulate_mouse_down(first.center(), MouseButton::Left, Modifiers::default());
    cx.executor().advance_clock(LIFT_DELAY);
    cx.run_until_parked();
    assert_eq!(phase(&view, cx), (true, true, true));
    let consumed = view.update(cx, |view, cx| view.release_workspace_drag(cx));
    assert!(consumed, "a lifted release is the drop");
    draw(cx);
    assert_eq!(phase(&view, cx), (false, false, false));
    assert_eq!(cx.debug_bounds(FIRST).unwrap(), first);
}

/// The preview lasts until the daemon's order replaces the one dropped on.
#[gpui::test]
fn the_settle_ends_when_the_daemon_reorders(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    let (first, _) = drop_first_before_third(&view, cx);
    // The second row holds the first's place while the drop is previewed.
    assert_eq!(cx.debug_bounds(SECOND).unwrap().top(), first.top());
    daemon_reorders(&view, cx);
    draw(cx);
    let (_, floating, previewing) = phase(&view, cx);
    assert!(!floating && !previewing);
    // The daemon's own order now puts the second row first, unshifted.
    assert_eq!(cx.debug_bounds(SECOND).unwrap().top(), first.top());
    cx.executor().advance_clock(SETTLE_TIMEOUT);
    cx.run_until_parked();
    assert_eq!(phase(&view, cx), (false, false, false));
}

/// A daemon that never answers leaves the rows where they were.
#[gpui::test]
fn the_settle_ends_when_the_daemon_never_answers(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    let (first, second) = drop_first_before_third(&view, cx);
    cx.executor().advance_clock(SETTLE_TIMEOUT);
    cx.run_until_parked();
    draw(cx);
    assert_eq!(phase(&view, cx), (false, false, false));
    assert_eq!(cx.debug_bounds(FIRST).unwrap(), first);
    assert_eq!(cx.debug_bounds(SECOND).unwrap(), second);
}

/// D1: a left release that arrives while the drop settles, a click anywhere
/// in the window, leaves the preview showing and is left to that click.
#[gpui::test]
fn a_release_during_the_settle_keeps_the_preview(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    let (first, _) = drop_first_before_third(&view, cx);
    assert_eq!(cx.debug_bounds(SECOND).unwrap().top(), first.top());

    let consumed = view.update(cx, |view, cx| view.release_workspace_drag(cx));
    assert!(!consumed, "a settling drag leaves the release to its click");
    draw(cx);
    assert_eq!(
        phase(&view, cx),
        (true, false, true),
        "the release ended the settling preview"
    );
    assert_eq!(cx.debug_bounds(SECOND).unwrap().top(), first.top());

    // The same through the window's own release handler.
    cx.simulate_mouse_up(
        point(px(700.), px(400.)),
        MouseButton::Left,
        Modifiers::default(),
    );
    draw(cx);
    assert_eq!(phase(&view, cx), (true, false, true));
    assert_eq!(cx.debug_bounds(SECOND).unwrap().top(), first.top());

    // The settle still ends on its own.
    cx.executor().advance_clock(SETTLE_TIMEOUT);
    cx.run_until_parked();
    assert_eq!(phase(&view, cx), (false, false, false));
}

/// Escape while the drop settles has nothing to put back: the move is sent.
#[gpui::test]
fn cancelling_during_the_settle_keeps_the_preview(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    drop_first_before_third(&view, cx);
    let carried = view.update(cx, |view, cx| view.cancel_workspace_drag(cx));
    assert!(!carried);
    assert_eq!(phase(&view, cx), (true, false, true));
}

/// D2: cancelling forgets a press that has not lifted yet, so its timer
/// cannot lift the row later, and leaves the key to the window.
#[gpui::test]
fn cancelling_before_the_lift_forgets_the_press(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    let first = cx.debug_bounds(FIRST).unwrap();
    cx.simulate_mouse_down(first.center(), MouseButton::Left, Modifiers::default());
    assert_eq!(phase(&view, cx), (true, false, false));
    let carried = view.update(cx, |view, cx| view.cancel_workspace_drag(cx));
    assert!(
        !carried,
        "nothing was carried, so the key is not the drag's"
    );
    cx.executor().advance_clock(LIFT_DELAY * 2);
    cx.run_until_parked();
    assert_eq!(
        phase(&view, cx),
        (false, false, false),
        "the cancelled press lifted (held, floating, previewing)"
    );
}

/// Cancelling a lifted row puts it back and takes the key.
#[gpui::test]
fn cancelling_a_lifted_row_puts_it_back(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    let first = cx.debug_bounds(FIRST).unwrap();
    cx.simulate_mouse_down(first.center(), MouseButton::Left, Modifiers::default());
    cx.executor().advance_clock(LIFT_DELAY);
    cx.run_until_parked();
    let carried = view.update(cx, |view, cx| view.cancel_workspace_drag(cx));
    assert!(carried);
    assert_eq!(phase(&view, cx), (false, false, false));
}

/// The daemon's order ends the settle at once: the drag is forgotten on the
/// tick that delivers it, so the next gesture need not wait out the timeout.
#[gpui::test]
fn the_daemon_s_order_forgets_the_drag_and_frees_the_next_press(cx: &mut TestAppContext) {
    let peer = MockPeer::new();
    let (view, cx) = connected(&peer, cx);
    drop_first_before_third(&view, cx);
    // The reordered snapshot arrives through the endpoint's inbox.
    view.update(cx, |view, cx| {
        let mut state = view.live.clone();
        let mut reordered = snapshot(40);
        reordered.workspaces[..2].rotate_left(1);
        state.snapshot = Some(Arc::new(reordered));
        state.dirty = true;
        *view.endpoints[0].connection.inbox.lock().unwrap() = state;
        view.poll_endpoints(cx);
        let first = &view.live.snapshot.as_ref().unwrap().workspaces[0];
        assert_eq!(first.workspace_id, "w1", "the poll delivered the order");
    });
    assert_eq!(
        phase(&view, cx),
        (false, false, false),
        "the settled drag outlived the daemon's order (held, floating, previewing)"
    );
    // A device header can be pressed right away.
    let pressed = view.update(cx, |view, cx| {
        view.endpoints.extend(["a", "b"].into_iter().map(remote));
        view.press_device(1, 0, point(px(20.), px(20.)), cx);
        view.device_drag.is_some()
    });
    assert!(pressed, "a device press was refused after the settle ended");
}
