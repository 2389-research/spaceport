// ABOUTME: Keeps actual host and WSL setup bodies visible after their owners reset.
// ABOUTME: Verifies departure with private fields and no runtime discovery.
#![allow(clippy::unwrap_used)]
use super::*;
use crate::menu::devices::{device_setup_departure_fixture, device_setup_long_fixture};
use gpui::{Entity, ScrollDelta, ScrollEvent, TestAppContext, VisualTestContext, size};

mod isolation;

fn hold_departure(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, _| {
            assert!(view.menu.device_setup.is_none());
            assert!(view.menu.wsl_setup.is_none());
            assert!(view.menu.wsl_remove.is_none());
            assert!(view.focus.is_focused(window));
            let later = motion::now() + std::time::Duration::from_secs(3600);
            view.menu.appear.observe(None, None, later);
            let leaving = view
                .menu
                .appear
                .leaving
                .as_mut()
                .expect("setup popup leaves");
            leaving.motion.set(true, later, true);
            leaving.motion.set(false, later, false);
        });
        window.refresh();
        window.draw(cx).clear(cx);
    });
}

fn dismiss(view: &Entity<SheprdWindow>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.menu.appear.live = true;
            view.dismiss_menu(window, cx);
        })
    });
    hold_departure(view, cx);
}

fn departure(page: Page, selector: &'static str, cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| device_setup_departure_fixture(page, window, cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let before = cx.debug_bounds(selector).expect("live setup body appears");
    dismiss(&view, cx);
    assert_eq!(cx.debug_bounds(selector), Some(before));
}

#[gpui::test]
fn add_device_body_leaves_after_setup_fields_drop(cx: &mut TestAppContext) {
    departure(Page::AddDevice, "device-setup-body", cx);
}

#[gpui::test]
fn add_wsl_body_leaves_after_setup_fields_drop(cx: &mut TestAppContext) {
    departure(Page::AddWsl, "wsl-setup-body", cx);
}

#[gpui::test]
fn remove_wsl_body_leaves_after_confirmation_owner_drops(cx: &mut TestAppContext) {
    departure(Page::RemoveWsl, "remove-wsl-submit", cx);
}

fn scroll_departure(
    page: Page,
    body: &'static str,
    selector: &'static str,
    delta: f32,
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|window, cx| device_setup_long_fixture(page, window, cx));
    cx.simulate_resize(size(px(360.), px(320.)));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let body_bounds = cx.debug_bounds(body).unwrap();
    let before = cx.debug_bounds(selector).unwrap();
    cx.simulate_event(ScrollEvent {
        position: body_bounds.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-delta))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let scrolled = cx.debug_bounds(selector).unwrap();
    assert!(scrolled.top() < before.top());
    assert!(body_bounds.contains(&scrolled.center()));
    let panel = cx.debug_bounds("menu-panel").unwrap();
    dismiss(&view, cx);
    assert_eq!(cx.debug_bounds("menu-panel"), Some(panel));
    assert_eq!(cx.debug_bounds(body), Some(body_bounds));
    assert_eq!(cx.debug_bounds(selector), Some(scrolled));
    if page == Page::AddWsl {
        assert!(cx.debug_bounds("wsl-distro-0").is_none());
    }
}

#[gpui::test]
fn core_setup_scroll_keeps_field_positions_after_input_drop(cx: &mut TestAppContext) {
    scroll_departure(
        Page::AddDevice,
        "device-setup-body",
        "device-setup-field-1",
        90.,
        cx,
    );
}

#[gpui::test]
fn wsl_setup_scroll_keeps_only_visible_distribution_positions(cx: &mut TestAppContext) {
    scroll_departure(Page::AddWsl, "wsl-setup-body", "wsl-distro-6", 200., cx);
}

#[gpui::test]
fn remove_wsl_shell_scroll_keeps_the_confirmation_buttons_visible(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| device_setup_long_fixture(Page::RemoveWsl, window, cx));
    cx.simulate_resize(size(px(360.), px(150.)));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let panel = cx.debug_bounds("menu-panel").unwrap();
    let before = cx.debug_bounds("remove-wsl-submit").unwrap();
    cx.simulate_event(ScrollEvent {
        position: panel.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-120.))),
        ..Default::default()
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let scrolled = cx.debug_bounds("remove-wsl-submit").unwrap();
    assert!(scrolled.top() < before.top());
    assert!(panel.contains(&scrolled.center()));
    dismiss(&view, cx);
    assert_eq!(cx.debug_bounds("menu-panel"), Some(panel));
    assert_eq!(cx.debug_bounds("remove-wsl-submit"), Some(scrolled));
}
