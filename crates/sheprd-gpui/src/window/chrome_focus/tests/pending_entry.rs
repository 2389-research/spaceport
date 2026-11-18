// ABOUTME: Keeps terminal input isolated while chrome focus waits for its first frame.
// ABOUTME: Cancels deferred focus when Escape returns keyboard ownership to the terminal.
#![allow(clippy::unwrap_used)]
use super::{fixture_window, full_draw};
use crate::{sidebar::layout_tests::run_frames, state::ConnectionStatus};
use gpui::{Entity, Keystroke, Modifiers, TestAppContext, VisualTestContext};
use std::time::Duration;

fn fixture(cx: &mut TestAppContext) -> (Entity<crate::SheprdWindow>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        window.activate_window();
        view.update(cx, |view, cx| {
            view.live.status = ConnectionStatus::Disconnected;
            window.focus(&view.focus, cx);
        });
        full_draw(window, cx).clear(cx);
        assert!(view.read(cx).focus.is_focused(window));
        assert!(view.read(cx).flash_shown().is_none());
    });
    (view, cx)
}

#[gpui::test]
fn tab_during_region_entry_does_not_reach_the_terminal(cx: &mut TestAppContext) {
    let (view, cx) = fixture(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.cycle_region(true, window, cx));
        window.dispatch_keystroke(Keystroke::parse("tab").unwrap(), cx);
        assert!(
            view.read(cx).flash_shown().is_none(),
            "Tab must not attempt terminal input during chrome entry"
        );
    });
}

#[gpui::test]
fn escape_cancels_region_entry_before_deferred_focus_runs(cx: &mut TestAppContext) {
    let (view, cx) = fixture(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.cycle_region(true, window, cx));
        window.dispatch_keystroke(Keystroke::parse("escape").unwrap(), cx);
        assert!(
            view.read(cx).chrome_region.is_none(),
            "Escape must cancel entry"
        );
        assert!(view.read(cx).focus.is_focused(window));
    });
    run_frames(cx, 4, Duration::from_millis(16));
    cx.update(|window, cx| {
        assert!(view.read(cx).chrome_region.is_none());
        assert!(
            view.read(cx).focus.is_focused(window),
            "cancelled entry stole focus later"
        );
    });
}

#[gpui::test]
fn region_entry_finishes_at_a_real_control(cx: &mut TestAppContext) {
    let (view, cx) = fixture(cx);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| view.cycle_region(true, window, cx));
        assert!(view.read(cx).chrome_entry_focus.is_focused(window));
    });
    run_frames(cx, 4, Duration::from_millis(16));
    cx.update(|window, cx| {
        let state = view.read(cx);
        assert!(state.chrome_region.is_some());
        assert!(!state.chrome_entry_focus.is_focused(window));
        assert!(!state.focus.is_focused(window));
        assert!(window.focused(cx).is_some());
    });
}

#[gpui::test]
fn a_new_focus_owner_is_not_replaced_by_deferred_region_entry(cx: &mut TestAppContext) {
    let (view, cx) = fixture(cx);
    let chosen = cx.update(|window, cx| {
        view.update(cx, |view, cx| view.cycle_region(true, window, cx));
        let chosen = view.read(cx).sidebar_search.focus(cx);
        window.focus(&chosen, cx);
        chosen
    });
    run_frames(cx, 4, Duration::from_millis(16));
    cx.update(|window, _| assert!(chosen.is_focused(window)));
}

#[gpui::test]
fn mouse_chrome_actions_do_not_take_terminal_focus(cx: &mut TestAppContext) {
    let (view, cx) = fixture(cx);
    let sort = cx.debug_bounds("agents-sort").unwrap().center();
    cx.simulate_click(sort, Modifiers::default());
    cx.update(|window, cx| {
        assert!(view.read(cx).focus.is_focused(window));
        assert!(!view.read(cx).chrome_entry_focus.is_focused(window));
        assert!(view.read(cx).chrome_region.is_none());
    });
}
