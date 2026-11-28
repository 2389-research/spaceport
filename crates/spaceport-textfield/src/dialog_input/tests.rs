#![allow(clippy::unwrap_used)]

use super::DialogInput;
use gpui::{Keystroke, OperatingSystem};

mod graphemes;
mod key_table;
mod word_keys;

/// A keystroke as this desktop spells it, parsed the way
/// `spaceport-keymap`'s `parse_keystroke` parses one: with the same host OS,
/// which is what a `secondary-` keystroke is read against.
fn key(name: &str) -> Keystroke {
    let os = if cfg!(target_os = "macos") {
        OperatingSystem::Mac
    } else if cfg!(target_os = "windows") {
        OperatingSystem::Windows
    } else if cfg!(target_os = "freebsd") {
        OperatingSystem::FreeBsd
    } else {
        OperatingSystem::Linux
    };
    Keystroke::parse_with_os(name, os).unwrap_or_else(|error| panic!("{name}: {error}"))
}

#[test]
fn utf16_replacement_and_composition_use_relative_selection() {
    let mut input = DialogInput::new("a\u{1f600}z".into());
    assert_eq!(input.range_from_utf16(1..3), 1..5);
    assert_eq!(input.to_utf16(1..5), 1..3);
    input.replace(Some(1..3), "\u{65e5}\u{1f600}", true, Some(1..3));
    assert_eq!(input.text, "a\u{65e5}\u{1f600}z");
    assert_eq!(input.selection(), 4..8);
    assert_eq!(input.marked, Some(1..8));
    input.replace(None, "\u{672c}", true, Some(1..1));
    assert_eq!(input.text, "a\u{672c}z");
    assert_eq!(input.selection(), 4..4);
    input.replace(None, "\u{65e5}\u{672c}", false, None);
    assert_eq!(input.text, "a\u{65e5}\u{672c}z");
    assert_eq!(input.selection(), 7..7);
    assert!(input.marked.is_none());
    assert_eq!(
        input.range_from_utf16(999..1000),
        input.text.len()..input.text.len()
    );
}

#[gpui::test]
fn editing_uses_graphemes_selection_clipboard_and_bounded_single_line_text(
    cx: &mut gpui::TestAppContext,
) {
    cx.update(|cx| {
        let mut input =
            DialogInput::new("e\u{301}\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}".into());
        input.key(&key("end"), cx);
        input.key(&key("backspace"), cx);
        assert_eq!(input.text, "e\u{301}");
        input.key(&key("shift-left"), cx);
        assert_eq!(input.selection(), 0..3);
        input.key(&key("cmd-x"), cx);
        assert_eq!(input.text, "");
        input.key(&key("cmd-v"), cx);
        assert_eq!(input.text, "e\u{301}");
        input.replace(None, "\n\r\tX", false, None);
        assert_eq!(input.text, "e\u{301}X");
        input.replace(None, &"x".repeat(20000), false, None);
        assert_eq!(input.text.len(), 4100);
    });
}

#[gpui::test]
fn cmd_backspace_and_cmd_delete_reach_the_line_ends(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut input = DialogInput::new("one two".into());
        input.key(&key("cmd-left"), cx);
        for _ in 0..3 {
            input.key(&key("right"), cx);
        }
        input.key(&key("cmd-delete"), cx);
        assert_eq!(input.text, "one");
        input.key(&key("left"), cx);
        input.key(&key("cmd-backspace"), cx);
        assert_eq!(input.text, "e");
        assert_eq!(input.selection(), 0..0);
    });
}
