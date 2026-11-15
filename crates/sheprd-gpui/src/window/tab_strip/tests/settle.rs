//! A dropped Herdr tab settles: the strip previews the order it asked the
//! daemon for until the daemon's order arrives, or the daemon never answers.
//! Nothing else ends the preview, and a press that never lifts stays a click.
use {
    super::*,
    crate::{browser::Pick, reorder::SETTLE_TIMEOUT, window::MockPeer},
};

/// Four tabs over a daemon that moves tabs, with `a` dropped before `c`, so
/// the move is sent and the strip previews `b a c d`. Returns where `a` and
/// `b` rested before the drop.
fn settling(
    cx: &mut TestAppContext,
) -> (
    Entity<SheprdWindow>,
    &mut VisualTestContext,
    MockPeer,
    [Bounds<Pixels>; 2],
) {
    let mut peer = MockPeer::advertising(&["tab.move"]);
    let (view, cx) = four_tabs(cx, true);
    view.update(cx, |view, _| {
        view.endpoints[0].connection.handle = Some(peer.client.handle.clone());
        *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
    });
    draw(cx);
    let a = cx.debug_bounds("tab-a").unwrap();
    let b = cx.debug_bounds("tab-b").unwrap();
    cx.simulate_mouse_down(a.center(), MouseButton::Left, Modifiers::default());
    let over = point(a.center().x + b.size.width / 2. + px(4.), a.center().y);
    for _ in 0..2 {
        cx.simulate_mouse_move(over, MouseButton::Left, Modifiers::default());
        draw(cx);
    }
    assert_eq!(target(&view, cx), Some((2, "before c".into())));
    cx.simulate_mouse_up(over, MouseButton::Left, Modifiers::default());
    draw(cx);
    let request = peer.request();
    assert_eq!(request["method"], "tab.move");
    assert_eq!(
        request["params"],
        serde_json::json!({"tab_id": "a", "insert_index": 2})
    );
    assert_previewing(&view, cx, a, b);
    (view, cx, peer, [a, b])
}

/// Whether the strip still shows the drop in place: `b` where `a` rested,
/// and `a` after it, with nothing carried.
fn assert_previewing(
    view: &Entity<SheprdWindow>,
    cx: &mut VisualTestContext,
    a: Bounds<Pixels>,
    b: Bounds<Pixels>,
) {
    draw(cx);
    assert!(
        view.read_with(cx, |view, _| view.tab_drag.is_some()),
        "the drop's preview ended"
    );
    let passed = cx.debug_bounds("tab-b").unwrap();
    assert!(
        (passed.left() - a.left()).abs() <= px(0.5),
        "tab b at {passed:?} left the preview, where it stands at {a:?}"
    );
    let dropped = cx.debug_bounds("tab-a").unwrap();
    assert!(
        (dropped.right() - b.right()).abs() <= px(0.5),
        "tab a at {dropped:?} left the preview, where it ends at {b:?}"
    );
    assert!(cx.debug_bounds("lifted-tab-a").is_none());
}

/// Gives the window the daemon's tabs in `order`.
fn daemon_order(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext, order: &[&str]) {
    view.update(cx, |view, cx| {
        let mut snapshot = (**view.live.snapshot.as_ref().unwrap()).clone();
        snapshot.tabs = order
            .iter()
            .map(|id| {
                snapshot
                    .tabs
                    .iter()
                    .find(|tab| tab.tab_id == *id)
                    .unwrap()
                    .clone()
            })
            .collect();
        view.live.snapshot = Some(Arc::new(snapshot));
        *view.endpoints[0].connection.inbox.lock().unwrap() = view.live.clone();
        cx.notify();
    });
}

fn shown(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) -> Option<Pick> {
    view.read_with(cx, |view, _| view.group_pick(view.group_slots()[0].id))
}

/// A click elsewhere while the daemon has not answered leaves the preview
/// up, and stays a click.
#[gpui::test]
fn a_stray_release_does_not_end_a_dropped_tab_s_preview(cx: &mut TestAppContext) {
    let (view, cx, _peer, [a, b]) = settling(cx);
    let stray = point(px(800.), px(400.));
    cx.simulate_mouse_move(stray, None, Modifiers::default());
    assert_previewing(&view, cx, a, b);
    cx.simulate_mouse_down(stray, MouseButton::Left, Modifiers::default());
    cx.simulate_mouse_up(stray, MouseButton::Left, Modifiers::default());
    assert_previewing(&view, cx, a, b);
    // The release is not the drag's, so whatever is under it gets its click.
    let taken =
        cx.update(|window, cx| view.update(cx, |view, cx| view.release_tab_drag(window, cx)));
    assert!(!taken);
    assert_previewing(&view, cx, a, b);
    // Escape puts back a carried tab, not one already dropped.
    assert!(!view.update(cx, |view, cx| view.cancel_tab_drag(cx)));
    assert_previewing(&view, cx, a, b);
}

#[gpui::test]
fn the_preview_ends_when_the_daemon_s_order_arrives(cx: &mut TestAppContext) {
    let (view, cx, _peer, [a, b]) = settling(cx);
    // Another change of the daemon's leaves the order it was sent from.
    daemon_order(&view, cx, &["a", "b", "c", "d"]);
    assert_previewing(&view, cx, a, b);
    daemon_order(&view, cx, &["b", "a", "c", "d"]);
    draw(cx);
    draw(cx);
    view.read_with(cx, |view, _| assert!(view.tab_drag.is_none()));
    let first = cx.debug_bounds("tab-b").unwrap();
    assert!((first.left() - a.left()).abs() <= px(0.5), "{first:?}");
    let second = cx.debug_bounds("tab-a").unwrap();
    assert!((second.right() - b.right()).abs() <= px(0.5), "{second:?}");
}

#[gpui::test]
fn the_preview_ends_when_the_daemon_never_answers(cx: &mut TestAppContext) {
    let (view, cx, _peer, [a, b]) = settling(cx);
    cx.executor().advance_clock(SETTLE_TIMEOUT / 2);
    cx.run_until_parked();
    assert_previewing(&view, cx, a, b);
    cx.executor().advance_clock(SETTLE_TIMEOUT);
    cx.run_until_parked();
    view.read_with(cx, |view, _| assert!(view.tab_drag.is_none()));
    draw(cx);
    draw(cx);
    assert_eq!(cx.debug_bounds("tab-a").unwrap(), a);
    assert_eq!(cx.debug_bounds("tab-b").unwrap(), b);
}

/// A press let go before it lifts is a click, which shows the tab; a press
/// held until it lifts is a drop where it stood, which shows nothing.
#[gpui::test]
fn a_release_is_a_click_only_before_the_tab_lifts(cx: &mut TestAppContext) {
    let (view, cx) = four_tabs(cx, true);
    let b = cx.debug_bounds("tab-b").unwrap();
    let c = cx.debug_bounds("tab-c").unwrap();
    assert_eq!(shown(&view, cx), Some(Pick::Herdr("a".into())));

    cx.simulate_mouse_down(b.center(), MouseButton::Left, Modifiers::default());
    let taken =
        cx.update(|window, cx| view.update(cx, |view, cx| view.release_tab_drag(window, cx)));
    assert!(!taken);
    view.read_with(cx, |view, _| assert!(view.tab_drag.is_none()));
    cx.simulate_mouse_up(b.center(), MouseButton::Left, Modifiers::default());
    assert_eq!(shown(&view, cx), Some(Pick::Herdr("b".into())));
    // With no drag armed, a release is nobody's.
    let taken =
        cx.update(|window, cx| view.update(cx, |view, cx| view.release_tab_drag(window, cx)));
    assert!(!taken);

    cx.simulate_mouse_down(c.center(), MouseButton::Left, Modifiers::default());
    cx.executor().advance_clock(crate::reorder::LIFT_DELAY);
    cx.run_until_parked();
    draw(cx);
    assert!(cx.debug_bounds("lifted-tab-c").is_some());
    assert_eq!(target(&view, cx), None);
    cx.simulate_mouse_up(c.center(), MouseButton::Left, Modifiers::default());
    draw(cx);
    view.read_with(cx, |view, _| assert!(view.tab_drag.is_none()));
    assert_eq!(shown(&view, cx), Some(Pick::Herdr("b".into())));
    assert_eq!(cx.debug_bounds("tab-c").unwrap(), c);
    assert!(cx.debug_bounds("lifted-tab-c").is_none());
}
