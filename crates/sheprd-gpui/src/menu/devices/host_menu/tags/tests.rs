// ABOUTME: Checks tag field cleanup and bounded departure row capture.
// ABOUTME: Uses in-memory tags instead of personal preference files.
#![allow(clippy::unwrap_used)]
use super::*;
use core::prelude::v1::test;

#[gpui::test]
fn tag_field_releases_on_dismissal(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| departure_fixture(Page::DeviceTags, window, cx));
    let weak = view.read_with(cx, |view, _| {
        view.menu
            .host
            .as_ref()
            .unwrap()
            .tags
            .as_ref()
            .unwrap()
            .input
            .downgrade()
    });
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
fn tag_capture_keeps_only_visible_rows_with_full_list_extent(cx: &mut TestAppContext) {
    let (view, cx) =
        cx.add_window_view(|window, cx| departure_fixture(Page::DeviceTags, window, cx));
    view.update(cx, |view, _| {
        let picker = view.menu.host.as_mut().unwrap().tags.as_mut().unwrap();
        picker.rows = (0..200).map(|index| format!("fixture-{index}")).collect();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let row_height = cx.debug_bounds("device-tags-row-0").unwrap().size.height;
    view.read_with(cx, |view, cx| {
        let host = view.menu.host.as_ref().unwrap();
        let paint = view.device_tags_paint(host, true, cx).unwrap();
        assert!(paint.rows.len() < host.tags.as_ref().unwrap().rows.len());
        assert_eq!(paint.before, px(0.));
        assert_eq!(
            paint.before + paint.after + row_height * paint.rows.len() as f32,
            host.tags.as_ref().unwrap().layout.get().size.height
        );
    });
}
