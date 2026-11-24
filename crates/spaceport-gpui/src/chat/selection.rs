//! Transcript text selection, independent of which virtualized rows are
//! mounted. The model is `spaceport_chat_core::selection`'s — the carets, the
//! range between them, the text a copy takes, and the document that text is
//! projected from — kept at the paths the rest of the chat reads it at, beside
//! the gestures the window makes with it.

pub(crate) use spaceport_chat_core::selection::{
    Caret, Cell, Document, Key, Region, Segment, Selection,
};
