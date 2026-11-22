// ABOUTME: Checks translated device picker and setup text under the pseudo-locale.
// ABOUTME: Opts into Core for the retained host setup scenarios.
//! The `en-XA` leak test (`crate::i18n::leak_tests`) for the device picker
//! and Add Device's SSH form: each draws only translated
//! messages, except for device and provider names and the target typed.
use super::*;
use crate::i18n::leak_tests::{assert_translated, draw, under_en_xa};
use crate::sidebar::layout_tests::fixture_core_window;
use gpui::{point, px, size};

/// Names the rail shows as they are: WSL distributions.
fn sources() -> Vec<String> {
    let names = ["WSL".to_owned()];
    names.into_iter().collect()
}

#[test]
fn the_device_picker_shows_only_messages_under_en_xa() {
    under_en_xa("device picker en-XA leak", |cx, recorder| {
        let (view, cx) = cx.add_window_view(fixture_core_window);
        cx.simulate_resize(size(px(1000.), px(800.)));
        draw(cx);
        recorder.take();
        view.update_in(cx, |view, window, cx| {
            if view.open_menu(window, cx) {
                view.menu.anchor = point(px(20.), px(700.));
                view.menu.page = Some(Page::Devices);
                view.menu.selected = Some(0);
            }
        });
        draw(cx);
        let labels: Vec<String> = view.read_with(cx, |view, _| {
            view.endpoints.iter().map(|e| e.label.clone()).collect()
        });
        assert_translated("Device picker", &recorder.take(), &labels);
    });
}

#[test]
fn the_ssh_form_shows_only_messages_under_en_xa() {
    under_en_xa("add device en-XA leak", |cx, recorder| {
        let (view, cx) = cx.add_window_view(fixture_core_window);
        cx.simulate_resize(size(px(1000.), px(900.)));
        draw(cx);
        recorder.take();
        view.update_in(cx, |view, window, cx| {
            view.open_add_device(window, cx);
        });
        draw(cx);
        let data: Vec<String> = sources().into_iter().chain(["default".into()]).collect();
        assert_translated("Add Device", &recorder.take(), &data);
    });
}
