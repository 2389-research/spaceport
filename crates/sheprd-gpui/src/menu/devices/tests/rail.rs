// ABOUTME: Checks device setup source choices and retained form input.
// ABOUTME: Uses an explicitly enabled Core window for host setup controls.
use super::*;
use crate::menu::devices::rail::{self, Source};
use crate::sidebar::layout_tests::fixture_core_window;
use gpui::{px, size};

#[gpui::test]
fn add_device_rail_offers_only_manual_hosts(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.simulate_resize(size(px(900.), px(700.)));
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_add_device(window, cx)));
    cx.run_until_parked();
    cx.update(|window, cx| crate::sidebar::layout_tests::full_draw(window, cx).clear(cx));
    assert!(cx.debug_bounds("add-device-source-catalog").is_none());
    assert!(cx.debug_bounds("add-device-source-container").is_none());
    let sources = cx.update(|_, cx| view.read(cx).rail_sources());
    if cfg!(windows) {
        assert_eq!(sources, [Source::Host, Source::Wsl]);
        assert!(cx.debug_bounds("add-device-source-host").is_some());
        assert!(cx.debug_bounds("add-device-source-wsl").is_some());
        assert!(rail::add_device_offer().contains("WSL"));
    } else {
        assert_eq!(sources, [Source::Host]);
        assert!(cx.debug_bounds("add-device-rail").is_none());
        assert!(cx.debug_bounds("add-device-source-wsl").is_none());
    }
    assert!(!rail::add_device_offer().to_lowercase().contains("cloud"));
}

#[gpui::test]
fn manual_ssh_form_keeps_its_target(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_core_window);
    cx.update(|window, cx| view.update(cx, |view, cx| view.open_add_device(window, cx)));
    cx.update(|_, cx| {
        let field = view.read(cx).menu.device_setup.as_ref().unwrap().fields[0].clone();
        field.update(cx, |field, cx| field.set_text_selected("me@build-box", cx));
    });
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.open_source(Source::Host, window, cx);
            assert_eq!(view.menu.page, Some(Page::AddDevice));
            assert_eq!(
                view.menu.device_setup.as_ref().unwrap().fields[0]
                    .read(cx)
                    .text(),
                "me@build-box"
            );
        });
    });
}
