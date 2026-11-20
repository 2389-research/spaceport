// ABOUTME: Checks gesture warnings over a real Devices tab rather than an unused terminal.
// ABOUTME: Keeps feedback within the content area when window chrome is visible or hidden.
use super::*;
use crate::window::Flash;

#[gpui::test]
fn gesture_warnings_show_over_the_devices_view(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    open(&view, cx);
    assert!(cx.debug_bounds("devices-tab").is_some());
    view.update(cx, |view, cx| {
        view.show_flash(Flash::warning("Could not copy this text"), cx);
    });
    draw(cx);
    let tab = cx
        .debug_bounds("devices-tab")
        .expect("Devices remains mounted");
    let flash = cx.debug_bounds("flash").expect("warning over Devices");
    assert!(flash.left() >= tab.left() && flash.right() <= tab.right());
    assert!(flash.top() >= tab.top() && flash.bottom() <= tab.bottom());
}

#[gpui::test]
fn bottom_feedback_stays_above_the_bottom_tab_strip(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    view.update(cx, |view, _| {
        view.settings.shared = Some(
            crate::herdr_settings::Settings::parse_text("[ui]\ntab_bar_position = 'bottom'")
                .unwrap(),
        );
        view.config.clipboard_toast.position = crate::config::ClipboardToastPosition::BottomRight;
    });
    open(&view, cx);
    view.update(cx, |view, cx| {
        view.show_flash(Flash::warning("Could not copy"), cx)
    });
    draw(cx);
    let tab = cx.debug_bounds("devices-tab").expect("Devices is mounted");
    let flash = cx.debug_bounds("flash").expect("warning is mounted");
    assert!(flash.bottom() <= tab.bottom(), "{flash:?}; content {tab:?}");
}
