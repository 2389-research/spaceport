//! One reorder at a time: a press on a workspace row and a press on a device
//! header never arm together, whichever comes first.

use super::*;
use crate::{
    reorder::LIFT_DELAY,
    sidebar::layout_tests::{fixture_window, full_draw, snapshot},
    state::ConnectionStatus,
};
use gpui::{Entity, Modifiers, MouseButton, TestAppContext, VisualTestContext, px, size};
use std::sync::Arc;

/// Local, selected and connected, with two more devices, so both a workspace
/// row and a device header have somewhere to go.
fn two_kinds(cx: &mut TestAppContext) -> (Entity<SheprdWindow>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(snapshot(3)));
        view.live.status = ConnectionStatus::Connected;
        view.endpoints.extend(["a", "b"].into_iter().map(remote));
        view
    });
    cx.simulate_resize(size(px(800.), px(900.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    (view, cx)
}

/// Which presses are armed: (workspace, device).
fn armed(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) -> (bool, bool) {
    view.read_with(cx, |view, _| {
        (view.workspace_drag.is_some(), view.device_drag.is_some())
    })
}

/// A second press with no release between, as when the release was lost.
#[gpui::test]
fn a_device_press_is_refused_while_a_workspace_press_is_armed(cx: &mut TestAppContext) {
    let (view, cx) = two_kinds(cx);
    let row = cx.debug_bounds("workspace-local-w0").unwrap();
    let header = cx.debug_bounds("host-ssh:a").unwrap();
    cx.simulate_mouse_down(row.center(), MouseButton::Left, Modifiers::default());
    assert_eq!(armed(&view, cx), (true, false));
    cx.simulate_mouse_down(header.center(), MouseButton::Left, Modifiers::default());
    assert_eq!(armed(&view, cx), (true, false));
    cx.executor().advance_clock(LIFT_DELAY);
    cx.run_until_parked();
    assert_eq!(armed(&view, cx), (true, false));
}

#[gpui::test]
fn a_workspace_press_is_refused_while_a_device_press_is_armed(cx: &mut TestAppContext) {
    let (view, cx) = two_kinds(cx);
    let row = cx.debug_bounds("workspace-local-w0").unwrap();
    let header = cx.debug_bounds("host-ssh:a").unwrap();
    cx.simulate_mouse_down(header.center(), MouseButton::Left, Modifiers::default());
    assert_eq!(armed(&view, cx), (false, true));
    cx.simulate_mouse_down(row.center(), MouseButton::Left, Modifiers::default());
    assert_eq!(
        armed(&view, cx),
        (false, true),
        "(workspace, device) presses armed together"
    );
    cx.executor().advance_clock(LIFT_DELAY);
    cx.run_until_parked();
    let floating = view.read_with(cx, |view, _| {
        view.workspace_drag
            .as_ref()
            .is_some_and(WorkspaceDrag::floating)
    });
    assert!(!floating, "a workspace row lifted beside a device header");
}
