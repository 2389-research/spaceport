//! The words `prepared` writes into its entries: a cut answer's note, the
//! labels around a file's two versions, a question's heading and its answer's,
//! the line naming a terminal, and how a measured turn reads. A crate below the
//! app has no messages (`docs/i18n.md`), so this names the shape only: the app
//! reads the words in the language it shows and hands them down. A prepared
//! body caches them, which is why a pane keeps one [`Labels`] and replaces it
//! when the language shown changes.

use spaceport_chat_protocol::TurnSummary;

/// The chat labels, in the language the app showed when they were read.
#[derive(Clone, Debug)]
pub struct Labels {
    /// What ends an answer cut to the size shown.
    pub answer_cut: String,
    /// The label over a file's newer version, on a line of its own.
    pub diff_after: String,
    /// The label over a file's older version, on a line of its own.
    pub diff_before: String,
    /// The label over a new file, on a line of its own.
    pub diff_new_file: String,
    /// The label over an answer, on a line of its own.
    pub question_answer: String,
    /// The heading over question `number`, counted from 1.
    pub question_number: fn(usize) -> String,
    /// What ends answers cut to the size kept.
    pub question_truncated: String,
    /// The line naming the program's terminal, by its id.
    pub terminal: fn(&str) -> String,
    /// The title a tool asking the user questions is shown under.
    pub tool_ask_user: String,
    /// How a turn ended, how long it took, and the tokens it used when the
    /// agent reports them.
    pub turn: fn(&TurnSummary) -> String,
}
