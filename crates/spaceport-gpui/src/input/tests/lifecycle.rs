//! The terminal's and the dialog's platform input handlers, which keep their
//! entity weakly, so neither holds the window open once it is closed.

use super::*;

/// The composition-and-release exercise, out of the `#[gpui::test]` body: the
/// macro expands its argument's whole AST, and this test's nested closures
/// overflowed the compiler's expansion stack on Linux.
fn exercise(
    view: &Entity<SpaceportWindow>,
    terminal: &mut TerminalInputHandler,
    dialog: &mut ViewInputHandler<SpaceportWindow>,
    window: &mut Window,
    cx: &mut App,
) {
    view.update(cx, |view, _| {
        view.menu.page = Some(crate::menu::Page::RenameTab);
        view.menu.input = Some(crate::dialog_input::DialogInput::new("old".into()));
    });
    dialog.replace_and_mark_text_in_range(None, "日本", Some(0..1), window, cx);
    assert_eq!(dialog.marked_text_range(window, cx), Some(0..2));
    dialog.replace_text_in_range(None, "語", window, cx);
    assert_eq!(view.read(cx).menu.input.as_ref().unwrap().text, "語");
    view.update(cx, |view, _| {
        view.menu.page = None;
        view.marked = "日本".into();
    });
    assert_eq!(terminal.marked_text_range(window, cx), Some(0..2));
    assert_eq!(
        terminal
            .selected_text_range(false, window, cx)
            .unwrap()
            .range,
        2..2
    );
    terminal.unmark_text(window, cx);
    assert_eq!(terminal.marked_text_range(window, cx), None);
    assert!(!terminal.apple_press_and_hold_enabled());
}

/// The terminal's and the dialog's platform input handlers, which keep their
/// entity weakly, so neither holds the window open once it is closed.
///
/// A plain std `#[test]`, spelled out in full: this module's `use super::*`
/// inherits input.rs's `use gpui::*`, whose glob brings gpui's own `test`
/// macro here - and expanding THAT macro overflowed Linux rustc's stack
/// whatever the body held. The context is built the way the macro would.
#[::std::prelude::v1::test]
fn terminal_and_dialog_handlers_do_not_keep_herdr_window_alive() {
    let mut cx = TestAppContext::build(
        TestDispatcher::new(0),
        Some("terminal_and_dialog_handlers_do_not_keep_herdr_window_alive"),
    );
    let (view, window) = cx.add_window_view(fixture_window);
    let weak = view.downgrade();
    let mut terminal = TerminalInputHandler::new(Bounds::default(), view.clone(), false);
    let mut dialog = ViewInputHandler::new(Bounds::default(), view.clone());
    window.update(|window, cx| {
        exercise(&view, &mut terminal, &mut dialog, window, cx);
    });
    drop(view);
    window.update(|window, cx| {
        assert_eq!(terminal.marked_text_range(window, cx), None);
        assert_eq!(dialog.marked_text_range(window, cx), None);
        terminal.replace_text_in_range(None, "late", window, cx);
        dialog.replace_text_in_range(None, "late", window, cx);
    });
    // Linux keeps a window's root view alive until the window closes; macOS
    // releases it when the last strong reference drops. Close the window so
    // the assertion measures the handler, not the platform's window policy.
    // The handler reads above need the live window, so they run before this.
    window.update(|window, _| window.remove_window());
    window.run_until_parked();
    assert!(weak.upgrade().is_none());
}
