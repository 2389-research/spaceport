// ABOUTME: Requires a fresh Enter before forgetting a WSL distribution.
// ABOUTME: Checks cancellation without touching a real saved distribution.
use super::*;

#[gpui::test]
fn held_enter_cannot_confirm_wsl_removal(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update_in(cx, |view, window, cx| {
        let distro = format!("spaceport-confirmation-test-{}", uuid::Uuid::new_v4());
        let id = format!("wsl:{distro}");
        view.open_remove_wsl(&id, Point::default(), window, cx);
        let mut enter = key("enter");
        enter.is_held = true;
        for _ in 0..3 {
            view.remove_wsl_key(&enter, window, cx);
            assert_eq!(view.menu.page, Some(Page::RemoveWsl));
            assert_eq!(view.menu.wsl_remove.as_deref(), Some(distro.as_str()));
            assert!(!view.menu.removing_devices.contains(&id));
        }
        view.remove_wsl_key(&key("escape"), window, cx);
        assert!(view.menu.page.is_none());
        assert!(!view.menu.removing_devices.contains(&id));
    });
}
