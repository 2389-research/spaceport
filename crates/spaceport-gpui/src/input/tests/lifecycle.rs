//! The terminal's and the dialog's platform input handlers, which keep their
//! entity weakly, so neither holds the window open once it is closed.

use super::*;

#[gpui::test]
fn terminal_and_dialog_handlers_do_not_keep_herdr_window_alive(cx: &mut TestAppContext) {
    let (_, cx) = cx.add_window_view(fixture_window);
    let view = cx.update(|window, cx| cx.new(|cx| fixture_window(window, cx)));
    let weak = view.downgrade();
    let mut terminal = TerminalInputHandler::new(Bounds::default(), view.clone(), false);
    let mut dialog = ViewInputHandler::new(Bounds::default(), view.clone());
    cx.update(|window, cx| {
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
    });
    drop(view);
    cx.run_until_parked();
    assert!(weak.upgrade().is_none());
    cx.update(|window, cx| {
        assert_eq!(terminal.marked_text_range(window, cx), None);
        assert_eq!(dialog.marked_text_range(window, cx), None);
        terminal.replace_text_in_range(None, "late", window, cx);
        dialog.replace_text_in_range(None, "late", window, cx);
    });
}
