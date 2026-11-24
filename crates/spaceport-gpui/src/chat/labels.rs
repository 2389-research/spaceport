//! The words `prepared` writes into its entries, read in the language the app
//! shows: a cut answer's note, the labels around a file's two versions, a
//! question's heading and its answer's, and the line naming a terminal. The
//! shape is [`spaceport_chat_core::labels`]'s; reading them is the app's work,
//! so they are read once and handed down. A prepared body caches them, which is
//! why a pane keeps one [`Labels`] and replaces it when the language shown
//! changes.

use crate::i18n::fl;
pub(crate) use spaceport_chat_core::labels::Labels;

/// The labels in the language the app shows now.
pub(crate) fn now() -> Labels {
    Labels {
        answer_cut: fl!("chat-answer-cut"),
        diff_after: fl!("chat-diff-after"),
        diff_before: fl!("chat-diff-before"),
        diff_new_file: fl!("chat-diff-new-file"),
        question_answer: fl!("chat-question-answer"),
        question_number: |number| fl!("chat-question-number", number = number),
        question_truncated: fl!("chat-question-truncated"),
        terminal: |id| fl!("chat-tool-terminal", id = id.to_owned()),
        tool_ask_user: fl!("chat-tool-ask-user"),
        turn: super::text::turn,
    }
}
