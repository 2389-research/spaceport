// ABOUTME: Extends prepared prose only after the session proves append-only changes.
// ABOUTME: Falls back for Markdown boundaries while preserving prior immutable bodies.
use super::{Body, MESSAGE, Part, message_end};
use crate::chat::rich::{Block, Inline, Kind, SourceTail, Style, code_id, within_word_limit};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub(super) enum Tail {
    // keep-sorted start
    Complete { bytes: usize, shown: usize },
    Cut { bytes: usize },
    // keep-sorted end
}

impl Tail {
    pub(super) fn of(text: &str) -> Self {
        if message_end(text) < text.len() {
            Self::Cut { bytes: text.len() }
        } else {
            Self::Complete {
                bytes: text.len(),
                shown: text.trim_end_matches(' ').len(),
            }
        }
    }
}

pub(super) fn extend(body: &Rc<Body>, tail: Tail, text: &str) -> Option<(Rc<Body>, Tail)> {
    let (bytes, shown) = match tail {
        Tail::Cut { bytes } => {
            text.get(..bytes)?;
            // Appended text cannot change the prefix the message limits already clipped.
            return Some((body.clone(), Tail::Cut { bytes: text.len() }));
        }
        Tail::Complete { bytes, shown } => (bytes, shown),
    };
    let raw = text.get(..bytes)?;
    if text.len() > MESSAGE.body || !within_word_limit(text) {
        return None;
    }
    let previous = text.get(..shown)?;
    let word = previous.rsplit(' ').next()?;
    if word.is_empty() || !word.chars().all(char::is_alphanumeric) {
        return None;
    }
    let end = text.trim_end_matches(' ').len();
    let added = text.get(shown..end)?;
    if !added
        .chars()
        .all(|character| character == ' ' || character.is_alphanumeric())
    {
        return None;
    }
    let Body::Message(parts) = &**body else {
        return None;
    };
    let [Part::Rich(blocks)] = &parts[..] else {
        return None;
    };
    let mut blocks = blocks.to_vec();
    // Definitions and excess table cells can follow visible text without producing text events.
    match blocks.last_mut()? {
        Block::Paragraph {
            kind,
            content,
            source_tail,
            ..
        } if !matches!(*kind, Kind::Heading(_) | Kind::Rule) => {
            let (suffix, added) = match source_tail {
                SourceTail::Trimmed => (word, added),
                SourceTail::Verbatim => (
                    raw.get(shown.checked_sub(word.len())?..)?,
                    text.get(bytes..)?,
                ),
                SourceTail::Unmapped => return None,
            };
            append_text(content, suffix, added)?;
        }
        Block::Table {
            rows,
            source_tail: SourceTail::Trimmed,
            ..
        } => append_text(rows.last_mut()?.last_mut()?, word, added)?,
        Block::Code {
            id,
            language,
            source,
        } => {
            if raw.ends_with(['\n', '\r']) {
                return None;
            }
            // Verify the current raw line belongs to this code, rather than an unfinished fence header.
            let last_line = source.rsplit('\n').next()?;
            if last_line.is_empty() || !raw.ends_with(last_line) {
                return None;
            }
            let mut extended = source.to_string();
            extended.push_str(text.get(bytes..)?);
            *source = extended.into();
            *id = code_id(language, source);
        }
        _ => return None,
    }
    Some((
        Rc::new(Body::Message(vec![Part::Rich(blocks.into())])),
        Tail::Complete {
            bytes: text.len(),
            shown: end,
        },
    ))
}

fn append_text(content: &mut [Inline], suffix: &str, added: &str) -> Option<()> {
    let Inline::Text(text, style) = content.last_mut()? else {
        return None;
    };
    if *style != Style::default() || !text.ends_with(suffix) {
        return None;
    }
    text.push_str(added);
    Some(())
}
