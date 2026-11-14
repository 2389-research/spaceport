//! Which edit a key asks of a text field. One table serves the search field
//! and the composer, which differ only in what a line is and whether words
//! are hidden; a dialog's draft keeps the narrower table it has always had.
//! Nothing here touches a field: its owner applies the command, and keeps
//! the clipboard, the kill buffer, and what a masked field must not reveal.

use std::ops::Range;

use gpui::Modifiers;

use super::Editing;
use crate::{controls::CTRL_SHORTCUTS, words};

/// The desktops' modifier conventions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Platform {
    /// Cmd edits, Option moves by word, and Control keeps its Emacs keys.
    MacOs,
    /// Linux and Windows: Ctrl edits as well as the platform key, and moves
    /// by word.
    CtrlDesktop,
    /// Any other desktop: the platform key edits, Ctrl moves by word.
    Other,
}

impl Platform {
    /// The desktop this build runs on.
    pub(crate) const CURRENT: Self = if cfg!(target_os = "macos") {
        Self::MacOs
    } else if CTRL_SHORTCUTS {
        Self::CtrlDesktop
    } else {
        Self::Other
    };

    /// Whether the modifiers are those of Cut, Copy, Paste, and Select All.
    /// The Ctrl desktops also take Ctrl-Shift, the terminal's alias for them.
    pub(crate) fn edit_modifiers(self, modifiers: Modifiers) -> bool {
        !modifiers.alt
            && !modifiers.function
            && ((modifiers.platform && !modifiers.control && !modifiers.shift)
                || (self == Self::CtrlDesktop && modifiers.control && !modifiers.platform))
    }

    /// Whether a field moves or deletes by word on an arrow, Backspace, or
    /// Delete: Option on macOS, Ctrl on the other desktops. Shift extends.
    pub(crate) fn word_modifiers(self, modifiers: Modifiers) -> bool {
        let (word, other) = match self {
            Self::MacOs => (modifiers.alt, modifiers.control),
            Self::CtrlDesktop | Self::Other => (modifiers.control, modifiers.alt),
        };
        word && !other && !modifiers.platform && !modifiers.function
    }

    /// Whether the modifiers are those of the Control keys every native
    /// macOS field takes from Emacs. The other desktops use Ctrl for Select
    /// All, Copy, and the rest, so there these keys keep that meaning.
    fn emacs_modifiers(self, modifiers: Modifiers) -> bool {
        self == Self::MacOs
            && modifiers.control
            && !modifiers.platform
            && !modifiers.alt
            && !modifiers.shift
            && !modifiers.function
    }
}

/// Which keys a field takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KeyTable {
    /// The search field's and the composer's: the Emacs keys on macOS, then
    /// the edit and platform keys, the plain ones, and the word ones. A key
    /// with any other modifiers is left to the owner.
    Field,
    /// A dialog draft's: no Emacs keys, and an arrow, Home, End, Backspace,
    /// or Delete acts whatever else is held.
    Dialog,
}

/// What a field's keys are read against.
pub(crate) struct Keys {
    /// The line the caret is on, which Home, End, and the platform key's
    /// arrows and deletions reach the ends of: the whole text in a
    /// single-line field.
    pub(crate) line: Range<usize>,
    /// The text is a secret: it is one word, so moving by word shows
    /// nothing of where its words are.
    pub(crate) words_hidden: bool,
    pub(crate) table: KeyTable,
    pub(crate) platform: Platform,
}

/// An edit a key asks for. Offsets are bytes of the field's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum EditCommand {
    /// Put the selection on the clipboard.
    Copy,
    /// Put the selection on the clipboard and delete it.
    Cut,
    /// Delete the range, which may be empty.
    Delete(Range<usize>),
    /// Delete the range, keeping it for `Yank`; nothing when it is empty.
    Kill(Range<usize>),
    /// Move the caret, the selection's start staying when `extend`.
    Move {
        to: usize,
        extend: bool,
    },
    /// Replace the selection with the clipboard's text.
    Paste,
    SelectAll,
    /// Replace the selection with what `Kill` last kept, if anything.
    Yank,
}

/// The edit `key` asks for under `modifiers`, or `None` for a key the field
/// leaves to its owner or to the platform's text input. A field that is
/// composing asks nothing: its input method takes every key.
pub(crate) fn edit_command(
    key: &str,
    modifiers: Modifiers,
    edit: &Editing,
    keys: &Keys,
) -> Option<EditCommand> {
    match keys.table {
        KeyTable::Field => field_command(key, modifiers, edit, keys),
        KeyTable::Dialog => dialog_command(key, modifiers, edit, keys),
    }
}

fn field_command(
    key: &str,
    modifiers: Modifiers,
    edit: &Editing,
    keys: &Keys,
) -> Option<EditCommand> {
    let backward = matches!(key, "left" | "backspace");
    let shift = modifiers.shift;
    if keys.platform.emacs_modifiers(modifiers)
        && let Some(command) = emacs_command(key, edit, &keys.line)
    {
        return Some(command);
    }
    let edits = keys.platform.edit_modifiers(modifiers) && matches!(key, "a" | "c" | "v" | "x");
    if edits || (modifiers.platform && !modifiers.control && !modifiers.alt) {
        return match key {
            // As in every macOS text field: Cmd-Left and Cmd-Right go to
            // the line's ends, Cmd-Backspace and Cmd-Delete delete to them.
            "left" | "right" => Some(EditCommand::Move {
                to: line_end(&keys.line, backward),
                extend: shift,
            }),
            "backspace" | "delete" if !shift => Some(delete(edit, line_end(&keys.line, backward))),
            _ if shift && !edits => None,
            _ => clipboard_command(key),
        };
    }
    if !modifiers.platform && !modifiers.control && !modifiers.alt {
        return match key {
            "backspace" | "delete" => Some(delete(edit, character(edit, backward))),
            "home" | "end" => Some(EditCommand::Move {
                to: line_end(&keys.line, key == "home"),
                extend: shift,
            }),
            "left" | "right" => Some(EditCommand::Move {
                to: selection_edge(edit, backward, shift)
                    .unwrap_or_else(|| character(edit, backward)),
                extend: shift,
            }),
            _ => None,
        };
    }
    if keys.platform.word_modifiers(modifiers) {
        // macOS sends these as commands, never text, so the platform's text
        // input would drop them.
        return word_command(key, shift, edit, keys);
    }
    // Option/dead-key text goes through the platform's text input.
    None
}

fn dialog_command(
    key: &str,
    modifiers: Modifiers,
    edit: &Editing,
    keys: &Keys,
) -> Option<EditCommand> {
    let backward = matches!(key, "left" | "backspace");
    let shift = modifiers.shift;
    if keys.platform.edit_modifiers(modifiers)
        && let Some(command) = clipboard_command(key)
    {
        return Some(command);
    }
    if keys.platform.word_modifiers(modifiers)
        && let Some(command) = word_command(key, shift, edit, keys)
    {
        return Some(command);
    }
    match key {
        "home" | "end" => Some(EditCommand::Move {
            to: line_end(&keys.line, key == "home"),
            extend: shift,
        }),
        // The platform key reaches the line's ends, as macOS fields do.
        "left" | "right" | "backspace" | "delete" if modifiers.platform => {
            let to = line_end(&keys.line, backward);
            Some(match key {
                "left" | "right" => EditCommand::Move { to, extend: shift },
                _ => delete(edit, to),
            })
        }
        "left" | "right" => Some(EditCommand::Move {
            to: selection_edge(edit, backward, shift).unwrap_or_else(|| character(edit, backward)),
            extend: shift,
        }),
        "backspace" | "delete" => Some(delete(edit, character(edit, backward))),
        _ => None,
    }
}

/// Ctrl-A and Ctrl-E to the line's ends, Ctrl-B and Ctrl-F a character back
/// and forward, Ctrl-H and Ctrl-D delete one, Ctrl-K cuts to the line's end,
/// and Ctrl-Y puts it back.
fn emacs_command(key: &str, edit: &Editing, line: &Range<usize>) -> Option<EditCommand> {
    let to = |to| EditCommand::Move { to, extend: false };
    Some(match key {
        "a" => to(line.start),
        "e" => to(line.end),
        "b" => to(edit.previous()),
        "f" => to(edit.next()),
        "h" => delete(edit, edit.previous()),
        "d" => delete(edit, edit.next()),
        // A selection is cut as it stands; otherwise the line's rest.
        "k" => {
            let selection = edit.selection();
            EditCommand::Kill(match selection.is_empty() {
                true => edit.cursor..line.end,
                false => selection,
            })
        }
        "y" => EditCommand::Yank,
        _ => return None,
    })
}

fn clipboard_command(key: &str) -> Option<EditCommand> {
    Some(match key {
        "a" => EditCommand::SelectAll,
        "c" => EditCommand::Copy,
        "v" => EditCommand::Paste,
        "x" => EditCommand::Cut,
        _ => return None,
    })
}

fn word_command(key: &str, shift: bool, edit: &Editing, keys: &Keys) -> Option<EditCommand> {
    let backward = matches!(key, "left" | "backspace");
    let word = |from| match (backward, keys.words_hidden) {
        (true, true) => 0,
        (false, true) => edit.text.len(),
        (true, false) => words::previous_start(&edit.text, from),
        (false, false) => words::next_end(&edit.text, from),
    };
    match key {
        // With a selection, a plain word arrow starts from its edge.
        "left" | "right" => Some(EditCommand::Move {
            to: word(selection_edge(edit, backward, shift).unwrap_or(edit.cursor)),
            extend: shift,
        }),
        "backspace" | "delete" => Some(delete(edit, word(edit.cursor))),
        _ => None,
    }
}

/// The edge of the selection a plain arrow collapses it to.
fn selection_edge(edit: &Editing, backward: bool, shift: bool) -> Option<usize> {
    let selection = edit.selection();
    (!shift && !selection.is_empty()).then_some(match backward {
        true => selection.start,
        false => selection.end,
    })
}

fn character(edit: &Editing, backward: bool) -> usize {
    match backward {
        true => edit.previous(),
        false => edit.next(),
    }
}

fn line_end(line: &Range<usize>, backward: bool) -> usize {
    match backward {
        true => line.start,
        false => line.end,
    }
}

/// Deletes the selection, or with none, from the caret to `to`.
fn delete(edit: &Editing, to: usize) -> EditCommand {
    let selection = edit.selection();
    EditCommand::Delete(match selection.is_empty() {
        true => edit.cursor.min(to)..edit.cursor.max(to),
        false => selection,
    })
}
