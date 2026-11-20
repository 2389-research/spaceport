// ABOUTME: Keeps each host popup's actual body visible after active state resets.
// ABOUTME: Uses fixed motion stamps instead of timing guesses during departure checks.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::menu::devices::{host_departure_fixture, host_long_tags_fixture};
use gpui::{Entity, ScrollDelta, ScrollEvent, TestAppContext, VisualTestContext, size};

fn leave(view: &Entity<SheprdWindow>, confirm: bool, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            if confirm {
                view.host_menu_key(&crate::actions::edit_key("enter"), window, cx);
                assert!(!view.menu.removing_devices.is_empty());
            } else {
                view.dismiss_menu(window, cx);
            }
            assert!(view.menu.host.is_none());
            assert!(view.focus.is_focused(window));
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view
                .menu
                .appear
                .leaving
                .as_mut()
                .expect("host popup leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn close(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    leave(view, false, cx);
}

fn departure(page: Page, selector: &'static str, cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| host_departure_fixture(page, window, cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx
        .debug_bounds(selector)
        .expect("the live host body appears");
    close(&view, cx);
    assert_eq!(
        cx.debug_bounds(selector),
        Some(before),
        "departing host body keeps its geometry"
    );
}

#[gpui::test]
fn host_actions_leave_after_the_host_drops(cx: &mut TestAppContext) {
    departure(Page::Host, "host-menu-0", cx);
}

#[gpui::test]
fn rename_device_leaves_after_the_host_drops(cx: &mut TestAppContext) {
    departure(Page::RenameDevice, "rename-device", cx);
}

#[cfg(unix)]
#[gpui::test]
fn forward_port_leaves_after_the_host_drops(cx: &mut TestAppContext) {
    departure(Page::ForwardPort, "forward-port", cx);
}

#[gpui::test]
fn device_tags_leave_after_the_host_drops(cx: &mut TestAppContext) {
    departure(Page::DeviceTags, "device-tags", cx);
}

#[gpui::test]
fn remove_device_leaves_after_the_host_drops(cx: &mut TestAppContext) {
    departure(Page::RemoveDevice, "remove-device-submit", cx);
}

#[gpui::test]
fn confirmed_removal_keeps_the_departing_confirmation_body(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| host_departure_fixture(Page::RemoveDevice, window, cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx.debug_bounds("remove-device-submit").unwrap();
    leave(&view, true, cx);
    assert_eq!(cx.debug_bounds("remove-device-submit"), Some(before));
}

#[gpui::test]
fn scrolled_host_forward_reasons_keep_their_width_and_position(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| host_departure_fixture(Page::Host, window, cx));
    cx.simulate_resize(size(px(600.), px(420.)));
    view.update(cx, |view, _| {
        for port in 3000..3016 {
            view.port_forwards.fixture(
                "penso@box",
                port,
                crate::port_forward::State::Ended(
                    "Private fixture ended with a long reason that wraps within this host popup."
                        .into(),
                ),
            );
        }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let panel = cx.debug_bounds("menu-panel").unwrap();
    let before = cx.debug_bounds("host-forward-3004").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-500.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let scrolled = cx.debug_bounds("host-forward-3004").unwrap();
    assert!(scrolled.top() < before.top());
    close(&view, cx);
    assert_eq!(cx.debug_bounds("menu-panel").unwrap().size, panel.size);
    assert_eq!(cx.debug_bounds("host-forward-3004"), Some(scrolled));
    view.read_with(cx, |view, _| {
        assert_eq!(view.port_forwards.for_host("penso@box").count(), 16)
    });
}

#[gpui::test]
fn scrolled_tag_rows_leave_without_retaining_offscreen_tags(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(host_long_tags_fixture);
    cx.simulate_resize(size(px(600.), px(420.)));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let panel = cx.debug_bounds("menu-panel").unwrap();
    let before = cx.debug_bounds("device-tags-row-10").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-300.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let scrolled = cx.debug_bounds("device-tags-row-10").unwrap();
    let body = cx.debug_bounds("device-tags").unwrap();
    let metrics = (
        cx.debug_bounds("device-tags-title"),
        cx.debug_bounds("device-tags-list"),
        cx.debug_bounds("theme-search"),
    );
    let offset = view.read_with(cx, |view, _| view.host_shell_scroll().unwrap().offset());
    assert!(scrolled.top() < before.top());
    close(&view, cx);
    assert_eq!(cx.debug_bounds("menu-panel").unwrap().size, panel.size);
    assert_eq!(
        cx.debug_bounds("device-tags-row-10"),
        Some(scrolled),
        "live body {body:?}, offset {offset:?}, live metrics {metrics:?}, frozen body {:?}, metrics {:?}",
        cx.debug_bounds("device-tags"),
        (
            cx.debug_bounds("device-tags-title"),
            cx.debug_bounds("device-tags-list"),
            cx.debug_bounds("device-tags-field-paint")
        )
    );
    assert_eq!(cx.debug_bounds("device-tags").unwrap().size, body.size);
    assert!(cx.debug_bounds("device-tags-row-0").is_none());
}
