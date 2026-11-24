//! What a pane program's OSC 52 copies do to the pasteboard: the text lands
//! there and the flash says where it came from, while an empty payload, which
//! asks to clear the clipboard, is dropped so it cannot wipe a copy the user
//! just made.
use crate::i18n::fl;
use gpui::ClipboardItem;
use spaceport_i18n::Locale;

/// A pane app's OSC 52 copy reaches the pasteboard and reports itself, unless
/// the toast is configured off.
#[gpui::test]
fn daemon_clipboard_writes_reach_the_pasteboard_and_report(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    view.update(cx, |view, cx| {
        assert!(view.apply_clipboard_writes(vec!["hello".into()], cx));
        assert!(view.flash.is_some(), "the copy is reported");
    });
    assert_eq!(
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
        Some("hello".into())
    );
    view.update(cx, |view, cx| {
        view.config.clipboard_toast.enabled = false;
        view.clear_flash(false);
        assert!(view.apply_clipboard_writes(vec!["quiet".into()], cx));
        assert!(view.flash.is_none(), "a disabled toast stays silent");
        assert!(!view.apply_clipboard_writes(Vec::new(), cx));
    });
    assert_eq!(
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
        Some("quiet".into())
    );
}

/// An empty OSC 52 payload is a program asking to clear the clipboard. It never
/// writes and never reports: the copy the user made a moment ago stays theirs,
/// and no flash claims a copy that did not happen.
#[gpui::test]
fn empty_clipboard_clears_never_write_or_report(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("my selection".into())));
    view.update(cx, |view, cx| {
        view.clear_flash(false);
        assert!(!view.apply_clipboard_writes(vec![String::new(), String::new()], cx));
        assert!(view.flash.is_none(), "a clear is not a copy");
    });
    assert_eq!(
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
        Some("my selection".into()),
        "the clear left the clipboard alone"
    );
}

/// A clear queued behind a copy in the same drain does not undo it, so the
/// text is written once and reported once, worded as the terminal's copy
/// rather than as the user's own selection.
#[gpui::test]
fn an_empty_clear_between_copies_leaves_the_text_and_one_flash(cx: &mut gpui::TestAppContext) {
    let (view, cx) = cx.add_window_view(crate::sidebar::layout_tests::fixture_window);
    cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string("my selection".into())));
    view.update(cx, |view, cx| {
        view.clear_flash(false);
        assert!(view.apply_clipboard_writes(vec!["copied by the pane".into(), String::new()], cx));
        let flash = view.flash.clone().map(|(flash, _)| flash);
        assert_eq!(
            flash.map(|flash| flash.text),
            Some(fl!("endpoint-copied").into()),
            "the copy reports itself, as the terminal's copy"
        );
    });
    assert_eq!(
        cx.update(|_, cx| cx.read_from_clipboard().and_then(|item| item.text())),
        Some("copied by the pane".into()),
        "the trailing clear did not wipe the copy"
    );
}

/// The terminal's copy is worded apart from the user's own selection copy, so a
/// flash never reads as a copy they did not make.
#[test]
fn a_terminal_copy_is_worded_apart_from_the_users_own_selection_copy() {
    crate::i18n::show(&Locale::FALLBACK);
    assert_eq!(fl!("endpoint-copied"), "copied from the terminal");
    assert_eq!(fl!("window-find-copied"), "copied to clipboard");
}
