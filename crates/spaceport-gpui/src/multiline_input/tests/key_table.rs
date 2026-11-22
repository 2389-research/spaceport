//! Every key the composer takes, pinned row by row: against
//! `key_table_single.txt` on the one-line text the single-line fields are
//! pinned with, and against `key_table_lines.txt` on three lines.
//! `crate::text_editing::key_matrix` lists the keys and says how to
//! regenerate the tables.

use super::*;
use crate::text_editing::key_matrix::{
    self, CLIPBOARD, LINES, LINES_STARTS, Outcome, SINGLE_STARTS, Start,
};
use gpui::EntityInputHandler;
use std::{cell::Cell, rc::Rc};

/// The field's owner, counting the keys the field lets through to it.
struct Owner {
    input: Entity<MultilineInput>,
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

fn owned(
    cx: &mut TestAppContext,
) -> (
    Entity<MultilineInput>,
    Rc<Cell<usize>>,
    &mut VisualTestContext,
) {
    let passed = Rc::new(Cell::new(0));
    let (owner, cx) = cx.add_window_view(|_, cx| Owner {
        input: cx.new(MultilineInput::new),
        passed: passed.clone(),
    });
    let input = owner.read_with(cx, |owner, _| owner.input.clone());
    cx.simulate_resize(size(px(700.), px(500.)));
    cx.update(|window, cx| {
        let focus = input.read(cx).focus.clone();
        window.focus(&focus, cx);
    });
    draw(cx);
    draw(cx);
    (input, passed, cx)
}

/// Starts a case: the text and selection, an empty kill buffer, the shared
/// clipboard, and the rows shaped so Up and Down have somewhere to go.
fn place(
    input: &Entity<MultilineInput>,
    text: &str,
    anchor: usize,
    cursor: usize,
    cx: &mut VisualTestContext,
) {
    cx.write_to_clipboard(gpui::ClipboardItem::new_string(CLIPBOARD.into()));
    input.update(cx, |input, cx| {
        input.edit = Editing {
            text: text.into(),
            anchor,
            cursor,
            marked: None,
        };
        input.killed.clear();
        input.goal_x = None;
        cx.notify();
    });
    draw(cx);
}

fn selection(input: &Entity<MultilineInput>, cx: &VisualTestContext) -> (usize, usize) {
    input.read_with(cx, |input, _| (input.edit.anchor, input.edit.cursor))
}

fn table(starts: &[Start], vertical: bool, cx: &mut TestAppContext) -> Vec<String> {
    let (input, passed, cx) = owned(cx);
    let mut rows = Vec::new();
    for start in starts {
        for keys in key_matrix::keys(vertical) {
            place(&input, start.text, start.anchor, start.cursor, cx);
            passed.set(0);
            cx.simulate_keystrokes(&key_matrix::keystrokes(keys));
            let (anchor, cursor) = selection(&input, cx);
            let outcome = Outcome {
                text: input.read_with(cx, |input, _| input.edit.text.clone()),
                anchor,
                cursor,
                passed: passed.get(),
                clipboard: cx.read_from_clipboard().and_then(|item| item.text()),
            };
            rows.push(key_matrix::row(start, keys, &outcome));
        }
    }
    rows
}

#[gpui::test]
fn every_key_does_what_the_single_line_table_pins(cx: &mut TestAppContext) {
    key_matrix::assert_pinned(
        "multiline_input/tests",
        "key_table_single.txt",
        include_str!("key_table_single.txt"),
        &table(&SINGLE_STARTS, true, cx),
    );
}

#[gpui::test]
fn every_key_does_what_the_three_line_table_pins(cx: &mut TestAppContext) {
    key_matrix::assert_pinned(
        "multiline_input/tests",
        "key_table_lines.txt",
        include_str!("key_table_lines.txt"),
        &table(&LINES_STARTS, false, cx),
    );
}

/// From a line's start the caret's column is the left edge whatever the
/// glyphs measure, so these rows hold on every platform's text system.
#[gpui::test]
fn up_and_down_move_between_lines_and_pass_at_the_first_and_last(cx: &mut TestAppContext) {
    let (input, passed, cx) = owned(cx);
    place(&input, LINES, 0, 0, cx);
    let press = |keys: &str, cx: &mut VisualTestContext| {
        passed.set(0);
        cx.simulate_keystrokes(keys);
        draw(cx);
        (selection(&input, cx), passed.get())
    };
    assert_eq!(press("up", cx), ((0, 0), 1));
    assert_eq!(press("down", cx), ((8, 8), 0));
    assert_eq!(press("down", cx), ((17, 17), 0));
    assert_eq!(press("down", cx), ((17, 17), 1));
    assert_eq!(press("shift-up", cx), ((17, 8), 0));
    assert_eq!(press("shift-up", cx), ((17, 0), 0));
    assert_eq!(press("shift-up", cx), ((17, 0), 1));
    assert_eq!(press("down", cx), ((8, 8), 0));
    assert_eq!(press("cmd-up", cx), ((0, 0), 0));
}

#[gpui::test]
fn a_composing_field_leaves_every_key_to_the_input_method(cx: &mut TestAppContext) {
    let (input, passed, cx) = owned(cx);
    place(&input, "ab\ncd", 2, 2, cx);
    cx.update(|window, cx| {
        input.update(cx, |input, cx| {
            input.replace_and_mark_text_in_range(None, "\u{65e5}", Some(1..1), window, cx);
        });
    });
    draw(cx);
    for keys in key_matrix::keys(true) {
        passed.set(0);
        let keystrokes = key_matrix::keystrokes(keys);
        cx.simulate_keystrokes(&keystrokes);
        input.read_with(cx, |input, _| {
            assert_eq!(input.edit.text, "ab\u{65e5}\ncd", "{keys}");
            assert_eq!((input.edit.anchor, input.edit.cursor), (5, 5), "{keys}");
            assert_eq!(input.edit.marked, Some(2..5), "{keys}");
        });
        assert_eq!(passed.get(), keystrokes.split(' ').count(), "{keys}");
    }
}
