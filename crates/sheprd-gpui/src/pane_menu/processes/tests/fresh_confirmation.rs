// ABOUTME: Requires a fresh Enter before sending selected process identities to the worker.
// ABOUTME: Keeps held navigation working on the process list.
use super::*;

fn key(name: &'static str, held: bool) -> KeyDownEvent {
    KeyDownEvent {
        keystroke: Keystroke::parse(name).unwrap(),
        is_held: held,
        prefer_character_input: false,
    }
}

#[gpui::test]
fn held_enter_cannot_confirm_process_termination(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = crate::sidebar::layout_tests::fixture_window(window, cx);
        view.live.snapshot = Some(Arc::new(crate::pane_menu::tests::snapshot()));
        view
    });
    let (watch, updates, commands) = Watch::fake();
    view.update_in(cx, |view, window, cx| {
        view.open_pane_menu("inactive", Point::default(), window, cx);
        view.show_pane_processes(watch, cx);
        updates.send(listing(&[(10, 0), (20, 1)])).unwrap();
        view.poll_pane_processes(cx);
        view.pane_menu_key(&key("down", true), window, cx);
        view.pane_menu_key(&key("down", true), window, cx);
        view.pane_menu_key(&key("space", false), window, cx);
        view.pane_menu_key(&key("enter", false), window, cx);
        assert_eq!(view.menu.page, Some(Page::KillProcesses));
        for _ in 0..3 {
            view.pane_menu_key(&key("enter", true), window, cx);
            assert_eq!(view.menu.page, Some(Page::KillProcesses));
            assert!(commands.try_recv().is_err());
        }
        view.pane_menu_key(&key("enter", false), window, cx);
        assert_eq!(view.menu.page, Some(Page::PaneProcesses));
        assert_eq!(
            commands.try_recv().unwrap(),
            Command::Kill(vec![Identity {
                pid: 20,
                started: 1
            }])
        );
        assert!(commands.try_recv().is_err());
    });
}
