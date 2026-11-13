// ABOUTME: Keeps the Devices overview usable while hiding Core-only setup controls.
// ABOUTME: Verifies disabled Core does not expose an Add Device action.
use super::*;

#[gpui::test]
fn devices_overview_omits_core_setup_when_disabled(cx: &mut gpui::TestAppContext) {
    let (view, cx) = window(cx);
    open(&view, cx);
    assert!(cx.debug_bounds("devices-tab").is_some());
    assert!(cx.debug_bounds("devices-add").is_none());
}
