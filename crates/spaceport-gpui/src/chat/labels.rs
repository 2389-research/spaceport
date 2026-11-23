//! The words `prepared` writes into its entries: a cut answer's note, the
//! labels around a file's two versions, a question's heading and its answer's,
//! and the line naming a terminal. Reading them is the app's work, so they are
//! read once here and handed down: a prepared body caches them, which is why a
//! pane keeps one [`Labels`] and replaces it when the language shown changes.

use crate::i18n::fl;

/// The chat labels, in the language the app showed when they were read.
#[derive(Clone, Debug)]
pub(crate) struct Labels {
    /// What ends an answer cut to the size shown.
    pub(crate) answer_cut: String,
    /// The label over a file's newer version, on a line of its own.
    pub(crate) diff_after: String,
    /// The label over a file's older version, on a line of its own.
    pub(crate) diff_before: String,
    /// The label over a new file, on a line of its own.
    pub(crate) diff_new_file: String,
    /// The label over an answer, on a line of its own.
    pub(crate) question_answer: String,
    /// The heading over question `number`, counted from 1.
    pub(crate) question_number: fn(usize) -> String,
    /// What ends answers cut to the size kept.
    pub(crate) question_truncated: String,
    /// The line naming the program's terminal, by its id.
    pub(crate) terminal: fn(&str) -> String,
    /// The title a tool asking the user questions is shown under.
    pub(crate) tool_ask_user: String,
}

impl Labels {
    /// Reads the labels in the language the app shows now.
    pub(crate) fn now() -> Self {
        Self {
            answer_cut: fl!("chat-answer-cut"),
            diff_after: fl!("chat-diff-after"),
            diff_before: fl!("chat-diff-before"),
            diff_new_file: fl!("chat-diff-new-file"),
            question_answer: fl!("chat-question-answer"),
            question_number: |number| fl!("chat-question-number", number = number),
            question_truncated: fl!("chat-question-truncated"),
            terminal: |id| fl!("chat-tool-terminal", id = id.to_owned()),
            tool_ask_user: fl!("chat-tool-ask-user"),
        }
    }
}
