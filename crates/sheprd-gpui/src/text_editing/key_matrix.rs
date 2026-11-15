//! The keys every text field's pinned key table is driven with, so the
//! tables of the search field, the composer, and the dialog draft can be read
//! side by side: one row per starting caret and key, in each widget's own
//! `tests/key_table*.txt`.
//!
//! Regenerate a table on macOS, where every row runs, with
//! `UPDATE_KEY_TABLES=1 cargo test -p sheprd-gpui key_table`, and review the
//! diff: a changed row is a changed key.

#![allow(clippy::unwrap_used)]

use std::fmt::Write as _;

/// A field's text with its selection: `anchor` is where it started, `cursor`
/// where the caret is.
pub(crate) struct Start {
    pub(crate) name: &'static str,
    pub(crate) text: &'static str,
    pub(crate) anchor: usize,
    pub(crate) cursor: usize,
}

/// `one é😀 two`: the accent is a combining mark (bytes 4..7 are one
/// character) and the emoji takes four bytes (7..11).
pub(crate) const SINGLE: &str = "one e\u{301}\u{1f600} two";

/// Three lines: `one é`, `😀 two`, `three`, the breaks at bytes 7 and 16.
pub(crate) const LINES: &str = "one e\u{301}\n\u{1f600} two\nthree";

pub(crate) const SINGLE_STARTS: [Start; 5] = [
    start("start", SINGLE, 0, 0),
    start("middle", SINGLE, 7, 7),
    start("end", SINGLE, 15, 15),
    start("selected", SINGLE, 4, 11),
    start("reversed", SINGLE, 11, 4),
];

pub(crate) const LINES_STARTS: [Start; 5] = [
    start("start", LINES, 0, 0),
    start("middle", LINES, 12, 12),
    start("end", LINES, 22, 22),
    start("selected", LINES, 4, 13),
    start("reversed", LINES, 13, 4),
];

const fn start(name: &'static str, text: &'static str, anchor: usize, cursor: usize) -> Start {
    Start {
        name,
        text,
        anchor,
        cursor,
    }
}

/// What every case starts with on the clipboard: a tab and a line break show
/// what each field keeps of a paste.
pub(crate) const CLIPBOARD: &str = "P\tQ\r\nR";

/// Keys pressed on every platform, Enter aside: the test platform types an
/// unhandled Enter as a line break, which is not the field's doing. `word-` is the modifier fields move by
/// word with and `edit-` the one of Cut, Copy, Paste, and Select All; `cmd-`
/// is the platform key everywhere.
const KEYS: &[&str] = &[
    "left",
    "right",
    "home",
    "end",
    "up",
    "down",
    "backspace",
    "delete",
    "escape",
    "shift-left",
    "shift-right",
    "shift-home",
    "shift-end",
    "shift-up",
    "shift-down",
    "shift-backspace",
    "shift-delete",
    "cmd-left",
    "cmd-right",
    "cmd-home",
    "cmd-end",
    "cmd-up",
    "cmd-down",
    "cmd-backspace",
    "cmd-delete",
    "cmd-shift-left",
    "cmd-shift-right",
    "cmd-shift-backspace",
    "cmd-shift-delete",
    "cmd-shift-a",
    "alt-cmd-left",
    "alt-cmd-backspace",
    "word-left",
    "word-right",
    "word-backspace",
    "word-delete",
    "word-shift-left",
    "word-shift-right",
    "word-shift-backspace",
    "word-shift-delete",
    "edit-a",
    "edit-c",
    "edit-x",
    "edit-v",
];

/// Keys pressed on macOS alone: there Control is neither the word nor the
/// edit modifier, so these rows mean something else on the other desktops.
const MACOS_KEYS: &[&str] = &[
    "ctrl-a",
    "ctrl-e",
    "ctrl-b",
    "ctrl-f",
    "ctrl-h",
    "ctrl-d",
    "ctrl-k",
    "ctrl-y",
    "ctrl-k ctrl-y",
    "ctrl-k ctrl-a ctrl-y",
    "ctrl-k ctrl-e ctrl-k ctrl-y",
    "ctrl-shift-a",
    "ctrl-shift-e",
    "ctrl-cmd-a",
    "ctrl-left",
    "ctrl-right",
    "ctrl-home",
    "ctrl-backspace",
    "ctrl-delete",
];

/// The rows this platform runs. `vertical` is false for a text of several
/// lines, where Up and Down land by glyph widths the platforms disagree on.
pub(crate) fn keys(vertical: bool) -> impl Iterator<Item = &'static str> {
    let macos: &[&str] = if cfg!(target_os = "macos") {
        MACOS_KEYS
    } else {
        &[]
    };
    KEYS.iter()
        .chain(macos)
        .copied()
        .filter(move |keys| vertical || !(keys.ends_with("up") || keys.ends_with("down")))
}

/// The keystrokes a row's keys are on this platform, separated by spaces.
pub(crate) fn keystrokes(keys: &str) -> String {
    let (word, edit) = if cfg!(target_os = "macos") {
        ("alt-", "cmd-")
    } else {
        ("ctrl-", "ctrl-")
    };
    keys.replace("word-", word).replace("edit-", edit)
}

fn escaped(text: &str) -> String {
    text.chars().fold(String::new(), |mut out, ch| {
        if ch.is_ascii_graphic() || ch == ' ' {
            out.push(ch);
        } else {
            let _ = write!(out, "{}", ch.escape_unicode());
        }
        out
    })
}

/// What a case left behind.
pub(crate) struct Outcome {
    pub(crate) text: String,
    pub(crate) anchor: usize,
    pub(crate) cursor: usize,
    /// How many of the keys the field let through to its owner.
    pub(crate) passed: usize,
    pub(crate) clipboard: Option<String>,
}

/// One row of a table.
pub(crate) fn row(start: &Start, keys: &str, outcome: &Outcome) -> String {
    let clipboard = match outcome.clipboard.as_deref() {
        Some(CLIPBOARD) => "kept".to_owned(),
        Some(text) => format!("\"{}\"", escaped(text)),
        None => "none".to_owned(),
    };
    format!(
        "{} | {} => \"{}\" {}..{} passed={} clipboard={}",
        start.name,
        keys,
        escaped(&outcome.text),
        outcome.anchor,
        outcome.cursor,
        outcome.passed,
        clipboard,
    )
}

fn id(row: &str) -> &str {
    row.split(" => ").next().unwrap_or(row)
}

/// Compares the rows a widget produced with its pinned table, the file
/// `name` beside the test. Rows pinned for another platform are skipped.
#[track_caller]
pub(crate) fn assert_pinned(directory: &str, name: &str, pinned: &str, rows: &[String]) {
    if std::env::var_os("UPDATE_KEY_TABLES").is_some() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join(directory)
            .join(name);
        std::fs::write(&path, rows.join("\n") + "\n").unwrap();
        return;
    }
    let pinned: std::collections::HashMap<&str, &str> =
        pinned.lines().map(|line| (id(line), line)).collect();
    let changed: Vec<String> = rows
        .iter()
        .filter(|row| pinned.get(id(row)) != Some(&row.as_str()))
        .map(|row| {
            format!(
                "  pinned: {}\n  now:    {row}",
                pinned.get(id(row)).unwrap_or(&"(no such row)")
            )
        })
        .collect();
    assert!(
        changed.is_empty(),
        "{} of {} keys in {name} changed:\n{}",
        changed.len(),
        rows.len(),
        changed.join("\n")
    );
}
