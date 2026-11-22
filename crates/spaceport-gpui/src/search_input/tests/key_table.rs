#![allow(clippy::unwrap_used)]

//! Every key the search field takes, pinned row by row against
//! `key_table_plain.txt`, and against `key_table_masked.txt` for a field that
//! hides a secret. `crate::text_editing::key_matrix` lists the keys and says
//! how to regenerate the tables.

use super::*;
use crate::text_editing::key_matrix::{self, CLIPBOARD, Outcome, SINGLE_STARTS};
use gpui::{Entity, VisualTestContext};
use std::{cell::Cell, rc::Rc};

/// The field's owner, counting the keys the field lets through to it.
struct Owner {
    input: Entity<SearchInput>,
    passed: Rc<Cell<usize>>,
}

impl Render for Owner {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let passed = self.passed.clone();
        div()
            .on_key_down(move |_, _, _| passed.set(passed.get() + 1))
            .child(self.input.clone())
    }
}

fn field(
    masked: bool,
    cx: &mut gpui::TestAppContext,
) -> (Entity<SearchInput>, Rc<Cell<usize>>, &mut VisualTestContext) {
    let passed = Rc::new(Cell::new(0));
    let (owner, cx) = cx.add_window_view(|_, cx| Owner {
        input: cx.new(SearchInput::new),
        passed: passed.clone(),
    });
    let input = owner.read_with(cx, |owner, _| owner.input.clone());
    cx.update(|window, cx| {
        input.update(cx, |input, cx| input.set_masked(masked, cx));
        window.focus(&input.focus_handle(cx), cx);
        window.draw(cx).clear(cx);
    });
    (input, passed, cx)
}

fn place(
    input: &Entity<SearchInput>,
    text: &str,
    anchor: usize,
    cursor: usize,
    cx: &mut VisualTestContext,
) {
    cx.write_to_clipboard(ClipboardItem::new_string(CLIPBOARD.into()));
    input.update(cx, |input, cx| {
        input.edit = Editing {
            text: text.into(),
            anchor,
            cursor,
            marked: None,
        };
        input.killed.clear();
        cx.notify();
    });
}

fn table(masked: bool, cx: &mut gpui::TestAppContext) -> Vec<String> {
    let (input, passed, cx) = field(masked, cx);
    let mut rows = Vec::new();
    for start in &SINGLE_STARTS {
        for keys in key_matrix::keys(true) {
            place(&input, start.text, start.anchor, start.cursor, cx);
            passed.set(0);
            cx.simulate_keystrokes(&key_matrix::keystrokes(keys));
            let outcome = input.read_with(cx, |input, _| Outcome {
                text: input.edit.text.clone(),
                anchor: input.edit.anchor,
                cursor: input.edit.cursor,
                passed: passed.get(),
                clipboard: None,
            });
            let outcome = Outcome {
                clipboard: cx.read_from_clipboard().and_then(|item| item.text()),
                ..outcome
            };
            rows.push(key_matrix::row(start, keys, &outcome));
        }
    }
    rows
}

#[gpui::test]
fn every_key_does_what_the_plain_table_pins(cx: &mut gpui::TestAppContext) {
    key_matrix::assert_pinned(
        "search_input/tests",
        "key_table_plain.txt",
        include_str!("key_table_plain.txt"),
        &table(false, cx),
    );
}

#[gpui::test]
fn every_key_does_what_the_masked_table_pins(cx: &mut gpui::TestAppContext) {
    key_matrix::assert_pinned(
        "search_input/tests",
        "key_table_masked.txt",
        include_str!("key_table_masked.txt"),
        &table(true, cx),
    );
}

#[gpui::test]
fn a_masked_secret_reaches_neither_the_clipboard_nor_the_kill_buffer(
    cx: &mut gpui::TestAppContext,
) {
    let (input, _, cx) = field(true, cx);
    let mut keys = vec!["edit-c", "edit-x"];
    if cfg!(target_os = "macos") {
        keys.extend(["ctrl-k", "ctrl-a ctrl-k"]);
    }
    for keys in keys {
        place(&input, "hunter2", 0, 7, cx);
        cx.simulate_keystrokes(&key_matrix::keystrokes(keys));
        assert_eq!(
            cx.read_from_clipboard()
                .and_then(|item| item.text())
                .as_deref(),
            Some(CLIPBOARD),
            "{keys} put the secret on the clipboard"
        );
        input.read_with(cx, |input, _| {
            assert!(
                input.killed.is_empty(),
                "{keys} kept the secret to put back"
            );
        });
    }
}

#[gpui::test]
fn a_composing_field_leaves_every_key_to_the_input_method(cx: &mut gpui::TestAppContext) {
    let (input, passed, cx) = field(false, cx);
    place(&input, "ab", 2, 2, cx);
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "\u{65e5}", Some(1..1), window, cx);
        });
    });
    for keys in key_matrix::keys(true) {
        passed.set(0);
        let keystrokes = key_matrix::keystrokes(keys);
        cx.simulate_keystrokes(&keystrokes);
        input.read_with(cx, |input, _| {
            assert_eq!(input.edit.text, "ab\u{65e5}", "{keys}");
            assert_eq!((input.edit.anchor, input.edit.cursor), (5, 5), "{keys}");
            assert_eq!(input.edit.marked, Some(2..5), "{keys}");
        });
        assert_eq!(passed.get(), keystrokes.split(' ').count(), "{keys}");
    }
}
