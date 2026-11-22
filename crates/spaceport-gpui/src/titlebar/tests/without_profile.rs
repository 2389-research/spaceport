// ABOUTME: Verifies window chrome has no profile controls or reserved account space.
// ABOUTME: Keeps local preferences reachable through the shared application actions.
use super::*;

#[gpui::test]
fn header_without_profile_uses_the_available_width(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(header_window);
    for width in [1200., 640., 320.] {
        cx.simulate_resize(size(px(width), px(600.)));
        cx.update(|window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        });
        assert!(cx.debug_bounds("titlebar-avatar").is_none());
        assert!(cx.debug_bounds("titlebar-account-slot").is_none());
        let center = cx.debug_bounds("titlebar-center").unwrap();
        assert!(center.right() <= px(width));
        #[cfg(target_os = "macos")]
        assert_eq!(center.right(), px(width));
        assert!(cx.debug_bounds("toggle-sidebar").is_some());
    }
}

#[gpui::test]
fn local_controls_open_without_profile(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(header_window);
    cx.update(|window, cx| {
        let focus = view.read(cx).focus.clone();
        focus.focus(window, cx);
        window.refresh();
        window.draw(cx).clear(cx);
    });
    for (command, destination) in [
        (crate::controls::Command::Keybinds, Page::Keybinds),
        (crate::controls::Command::Themes, Page::Themes),
        (crate::controls::Command::About, Page::About),
    ] {
        cx.update(|window, cx| {
            window.dispatch_action(Box::new(crate::RunCommand { command }), cx);
            window.refresh();
            window.draw(cx).clear(cx);
        });
        view.read_with(cx, |view, _| assert_eq!(view.menu.page, Some(destination)));
        cx.simulate_keystrokes("escape");
        view.read_with(cx, |view, _| assert!(view.menu.page.is_none()));
    }
    let before = cx.update(|_, cx| cx.windows().len());
    cx.update(|window, cx| {
        window.dispatch_action(
            Box::new(crate::RunCommand {
                command: crate::controls::Command::Settings,
            }),
            cx,
        );
    });
    cx.run_until_parked();
    assert_eq!(cx.update(|_, cx| cx.windows().len()), before + 1);
}
