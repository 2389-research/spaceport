//! Every key a dialog's draft takes, pinned row by row against
//! `key_table.txt`. `crate::text_editing::key_matrix` lists the keys and
//! says how to regenerate the table.

use super::*;
use crate::text_editing::key_matrix::{self, CLIPBOARD, Outcome, SINGLE_STARTS};
use gpui::ClipboardItem;

/// A draft holding `text` selected from `anchor` to the caret at `cursor`.
fn placed(text: &str, anchor: usize, cursor: usize) -> DialogInput {
    let mut input = DialogInput::new(text.into());
    input.anchor = anchor;
    input.cursor = cursor;
    input
}

/// Where the draft's selection started and where its caret is.
fn ends(input: &DialogInput) -> (usize, usize) {
    (input.anchor, input.cursor)
}

/// Presses `keys`; how many of them the draft left to its dialog.
fn press(input: &mut DialogInput, keys: &str, cx: &mut gpui::App) -> usize {
    key_matrix::keystrokes(keys)
        .split(' ')
        .filter(|key| !input.key(&crate::keymap::parse_keystroke(key).unwrap_or_default(), cx))
        .count()
}

#[gpui::test]
fn every_key_does_what_the_table_pins(cx: &mut gpui::TestAppContext) {
    let rows: Vec<String> = cx.update(|cx| {
        let mut rows = Vec::new();
        for start in &SINGLE_STARTS {
            for keys in key_matrix::keys(true) {
                cx.write_to_clipboard(ClipboardItem::new_string(CLIPBOARD.into()));
                let mut input = placed(start.text, start.anchor, start.cursor);
                let passed = press(&mut input, keys, cx);
                let (anchor, cursor) = ends(&input);
                let outcome = Outcome {
                    text: input.text.clone(),
                    anchor,
                    cursor,
                    passed,
                    clipboard: cx.read_from_clipboard().and_then(|item| item.text()),
                };
                rows.push(key_matrix::row(start, keys, &outcome));
            }
        }
        rows
    });
    key_matrix::assert_pinned(
        "dialog_input/tests",
        "key_table.txt",
        include_str!("key_table.txt"),
        &rows,
    );
}

#[gpui::test]
fn a_composing_draft_leaves_every_key_to_the_input_method(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let mut input = placed("ab", 2, 2);
        input.replace(None, "\u{65e5}", true, Some(1..1));
        for keys in key_matrix::keys(true) {
            let pressed = key_matrix::keystrokes(keys).split(' ').count();
            assert_eq!(press(&mut input, keys, cx), pressed, "{keys}");
            assert_eq!(input.text, "ab\u{65e5}", "{keys}");
            assert_eq!(ends(&input), (5, 5), "{keys}");
            assert_eq!(input.marked, Some(2..5), "{keys}");
        }
    });
}

#[gpui::test]
fn a_replacement_drops_control_characters_and_stops_at_its_limits(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        cx.write_to_clipboard(ClipboardItem::new_string("a\tb\r\nc\u{7f}\u{1b}d".into()));
        let mut input = placed("", 0, 0);
        assert_eq!(press(&mut input, "edit-v", cx), 0);
        assert_eq!((input.text.as_str(), ends(&input)), ("abcd", (4, 4)));
        // One replacement takes 4096 characters, and the draft 16384 bytes:
        // past that a replacement is refused whole, selection and all.
        for _ in 0..3 {
            input.replace(None, &"x".repeat(5000), false, None);
        }
        assert_eq!(input.text.len(), 4 + 3 * 4096);
        input.replace(Some(0..4), &"\u{e9}".repeat(4096), false, None);
        assert_eq!(input.text.len(), 4 + 3 * 4096);
        assert_eq!(ends(&input), (12292, 12292));
        input.replace(Some(0..2), &"y".repeat(4092), true, Some(1..3));
        assert_eq!(input.text.len(), 16382);
        assert_eq!(
            (ends(&input), input.marked.clone()),
            ((1, 3), Some(0..4092))
        );
        // An input method's backwards range is its start alone.
        input.unmark();
        let backwards = |start, end| Some(std::ops::Range { start, end });
        input.replace(backwards(6, 2), "z", true, backwards(1, 0));
        assert_eq!((ends(&input), input.marked.clone()), ((7, 7), Some(6..7)));
    });
}
