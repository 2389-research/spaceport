// ABOUTME: Exercises frozen host controls without the popup shell's input shield.
// ABOUTME: Keeps regressions inside synthetic profiles and handleless forwards.
#![allow(clippy::expect_used)]
use super::*;
use gpui::{Context, Div, Entity, IntoElement, Render, Window, div, prelude::*};

struct Frozen {
    source: Entity<crate::SheprdWindow>,
    paint: super::super::Paint,
}

impl Render for Frozen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body: Div = self
            .source
            .update(cx, |view, cx| view.render_host_paint(&self.paint, cx));
        div().w(px(500.)).child(body)
    }
}

fn frozen(page: Page, cx: &mut TestAppContext) -> (Entity<Frozen>, &mut VisualTestContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let source = cx.new(|cx| departure::fixture(page, window, cx));
        let paint = source.update(cx, |view, cx| {
            if page == Page::Host {
                view.port_forwards.fixture(
                    "penso@box",
                    3000,
                    crate::port_forward::State::Listening { local_port: 39876 },
                );
            }
            let paint = view.host_paint(cx).unwrap();
            view.dismiss_menu(window, cx);
            view.open_host_menu(HOST, point(px(20.), px(20.)), window, cx);
            match page {
                Page::RenameDevice => view.activate_host_menu(Action::Rename, window, cx),
                #[cfg(unix)]
                Page::ForwardPort => view.activate_host_menu(Action::ForwardPort, window, cx),
                Page::DeviceTags => view.activate_host_menu(
                    Action::Tags(super::super::tags::Mode::AlsoTagged),
                    window,
                    cx,
                ),
                Page::RemoveDevice => view.activate_host_menu(Action::Remove, window, cx),
                _ => {}
            }
            let host = view.menu.host.as_mut().unwrap();
            host.id = "private-replacement".into();
            if page == Page::RemoveDevice {
                host.saved = Saved::Local;
            }
            host.selected = Some(3);
            if let Some(input) = &host.input {
                input.update(cx, |input, cx| {
                    input.set_text_selected("invalid\u{7}value", cx)
                });
            }
            paint
        });
        Frozen { source, paint }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    (view, cx)
}

fn inert(page: Page, selectors: &[&'static str], cx: &mut TestAppContext) {
    let (frozen, cx) = frozen(page, cx);
    for &selector in selectors {
        let bounds = cx
            .debug_bounds(selector)
            .expect("frozen control remains visible");
        cx.simulate_mouse_move(bounds.center(), None, Modifiers::default());
        cx.simulate_click(bounds.center(), Modifiers::default());
    }
    frozen.read_with(cx, |frozen, cx| {
        let view = frozen.source.read(cx);
        let host = host(view);
        assert_eq!(view.menu.page, Some(page));
        assert_eq!(host.id, "private-replacement");
        assert_eq!(host.selected, Some(3));
        assert!(!host.renaming);
        assert!(host.error.is_none());
        assert!(view.menu.removing_devices.is_empty());
        assert!(!view.device_tags.any());
    });
}

#[gpui::test]
fn frozen_host_action_has_no_hover_or_click_effect(cx: &mut TestAppContext) {
    inert(Page::Host, &["host-menu-0"], cx);
}

#[gpui::test]
fn frozen_forward_controls_cannot_open_or_stop_a_replacement_hosts_tunnel(cx: &mut TestAppContext) {
    let (frozen, cx) = frozen(Page::Host, cx);
    for selector in ["host-forward-open-3000", "host-forward-stop-3000"] {
        let bounds = cx.debug_bounds(selector).unwrap();
        cx.simulate_click(bounds.center(), Modifiers::default());
    }
    assert!(cx.opened_url().is_none());
    frozen.read_with(cx, |frozen, cx| {
        let view = frozen.source.read(cx);
        assert_eq!(view.menu.page, Some(Page::Host));
        assert_eq!(host(view).id, "private-replacement");
        assert_eq!(view.port_forwards.for_host("penso@box").count(), 1);
    });
}

#[gpui::test]
fn frozen_rename_controls_cannot_submit_or_dismiss_replacement(cx: &mut TestAppContext) {
    inert(
        Page::RenameDevice,
        &["rename-device-submit", "rename-device-close"],
        cx,
    );
}

#[cfg(unix)]
#[gpui::test]
fn frozen_port_controls_cannot_submit_or_dismiss_replacement(cx: &mut TestAppContext) {
    inert(
        Page::ForwardPort,
        &["forward-port-submit", "forward-port-close"],
        cx,
    );
}

#[gpui::test]
fn frozen_tag_choices_cannot_change_replacement_preferences(cx: &mut TestAppContext) {
    inert(Page::DeviceTags, &["device-tags-row-0"], cx);
}

#[gpui::test]
fn frozen_removal_controls_cannot_confirm_or_dismiss_replacement(cx: &mut TestAppContext) {
    inert(
        Page::RemoveDevice,
        &["remove-device-submit", "remove-device-cancel"],
        cx,
    );
}

fn field_releases_on_dismissal(page: Page, cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| departure::fixture(page, window, cx));
    let weak = view.read_with(cx, |view, _| host(view).input.as_ref().unwrap().downgrade());
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
        assert!(weak.upgrade().unwrap().read(cx).focus.is_focused(window));
        view.update(cx, |view, cx| view.dismiss_menu(window, cx));
        assert!(view.read(cx).focus.is_focused(window));
        window.draw(cx).clear(cx);
    });
    assert!(weak.upgrade().is_none());
}

#[gpui::test]
fn rename_and_port_fields_release_on_dismissal(cx: &mut TestAppContext) {
    field_releases_on_dismissal(Page::RenameDevice, cx);
    #[cfg(unix)]
    field_releases_on_dismissal(Page::ForwardPort, cx);
}
