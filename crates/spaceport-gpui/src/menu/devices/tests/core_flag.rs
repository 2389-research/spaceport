// ABOUTME: Keeps Core-only device creation out of disabled windows.
// ABOUTME: Verifies explicit opt-in restores the existing device setup entry.
use super::*;

#[gpui::test]
fn disabled_core_hides_device_creation(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.endpoints[0].connection.target = spaceport_client::ConnectTarget::Local;
            assert_eq!(view.picker_lines().len(), view.endpoints.len() + 1);
            view.open_device_setup(window, cx);
            assert!(view.menu.page.is_none());
            assert!(view.menu.device_setup.is_none());
            view.open_add_device(window, cx);
            assert!(view.menu.page.is_none());
            assert!(view.menu.device_setup.is_none());
        });
    });
}

#[gpui::test]
fn enabled_core_keeps_existing_device_controls(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| crate::core_feature::install(true, cx));
    let (view, cx) = cx.add_window_view(fixture_window);
    cx.update(|window, cx| {
        view.update(cx, |view, cx| {
            view.endpoints[0].connection.target = spaceport_client::ConnectTarget::Local;
            assert_eq!(view.picker_lines().len(), view.endpoints.len() + 2);
            view.open_device_setup(window, cx);
            assert_eq!(view.menu.page, Some(Page::AddDevice));
        });
    });
}
