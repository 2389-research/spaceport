// ABOUTME: Keeps a held Enter from accepting saved-host removal.
// ABOUTME: Exercises the host menu's selection and confirmation handlers.
use super::*;
use gpui::{KeyDownEvent, Keystroke, Point};

#[gpui::test]
fn held_enter_cannot_confirm_device_removal(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(fixture_window);
    view.update_in(cx, |view, window, cx| {
        add_host(view);
        view.open_host_menu(HOST, Point::default(), window, cx);
        let remove = SSH_ACTIONS
            .iter()
            .position(|(action, _)| *action == Action::Remove)
            .unwrap();
        view.menu.host.as_mut().unwrap().selected = Some(remove);
        let mut enter = KeyDownEvent {
            keystroke: Keystroke::parse("enter").unwrap(),
            is_held: false,
            prefer_character_input: false,
        };
        view.host_menu_key(&enter, window, cx);
        assert_eq!(view.menu.page, Some(Page::RemoveDevice));
        // Avoid an external CLI even if this regression fires.
        view.menu.host.as_mut().unwrap().saved = Saved::Local;
        enter.is_held = true;
        for _ in 0..3 {
            view.host_menu_key(&enter, window, cx);
            assert_eq!(view.menu.page, Some(Page::RemoveDevice));
            assert!(!view.menu.removing_devices.contains(HOST));
        }
        enter.is_held = false;
        view.host_menu_key(&enter, window, cx);
        assert!(view.menu.page.is_none());
        assert!(view.menu.removing_devices.contains(HOST));
    });
}
