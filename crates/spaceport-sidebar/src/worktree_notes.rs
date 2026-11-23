//! Notes the user keeps on checkouts, to remember what each of many worktrees
//! is for or waiting on. They belong to this client alone: never sent to the
//! daemon nor written into the repository. They are keyed
//! by endpoint, repository, and branch, because workspace IDs do not survive
//! a daemon restart. Every window shows the same notes; the app keeps the file
//! they are saved in and the words a damaged one is reported with.
use serde::{Deserialize, Serialize};

/// Notes kept; the least recently edited go first beyond this.
pub const MAX_NOTES: usize = 512;
/// Characters in one note. A note is a reminder, not a document.
pub const MAX_CHARS: usize = 500;
/// Bytes in each part of a checkout's key. A longer key gets no note, so
/// every note the store accepts fits in the file it reads back.
const MAX_FIELD_BYTES: usize = 1024;
/// Room for the most notes the store keeps, each as large as it accepts:
/// JSON escapes at most double a string's bytes (quotes and backslashes are
/// one byte each), plus the field names and punctuation.
pub const MAX_FILE_BYTES: u64 =
    (MAX_NOTES * (2 * (MAX_CHARS * 4 + 3 * MAX_FIELD_BYTES) + 128) + 64) as u64;

/// The checkout a note is about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checkout {
    pub endpoint: String,
    pub repo_key: String,
    pub branch: String,
}

impl Checkout {
    fn valid(&self) -> bool {
        [&self.endpoint, &self.repo_key, &self.branch]
            .iter()
            .all(|field| !field.is_empty() && field.len() <= MAX_FIELD_BYTES)
    }

    fn is(&self, endpoint: &str, repo_key: &str, branch: &str) -> bool {
        self.endpoint == endpoint && self.repo_key == repo_key && self.branch == branch
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    #[serde(flatten)]
    pub checkout: Checkout,
    pub text: String,
}

impl Note {
    fn valid(&self) -> bool {
        self.checkout.valid() && clean(&self.text) == self.text && !self.text.is_empty()
    }
}

/// The notes a state file holds.
#[derive(Serialize, Deserialize)]
pub struct Saved {
    pub notes: Vec<Note>,
}

/// One line of bounded text, free of control and bidirectional formatting
/// characters, with its runs of whitespace collapsed.
pub fn clean(text: &str) -> String {
    let spaced: String = text
        .chars()
        .take(MAX_CHARS * 4)
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    spaceport_client::text::safe_text(&spaced, MAX_CHARS * 4)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_CHARS)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

/// The notes kept, oldest edit first.
#[derive(Default)]
pub struct Notes {
    notes: Vec<Note>,
    /// Counts changes, so a prepared list can tell it is out of date.
    revision: u64,
}

impl Notes {
    /// The notes in `saved`, in order, or none when there are more than the
    /// store keeps or one of them is not what it would have written.
    pub fn from_saved(saved: Vec<Note>) -> Option<Self> {
        (saved.len() <= MAX_NOTES && saved.iter().all(Note::valid)).then_some(Self {
            notes: saved,
            revision: 0,
        })
    }

    /// These notes as the file they are saved in holds them.
    pub fn saved(&self) -> Saved {
        Saved {
            notes: self.notes.clone(),
        }
    }

    pub fn get(&self, endpoint: &str, repo_key: &str, branch: &str) -> Option<&str> {
        self.notes
            .iter()
            .find(|note| note.checkout.is(endpoint, repo_key, branch))
            .map(|note| note.text.as_str())
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Every note, most recently edited first.
    pub fn recent(&self) -> impl Iterator<Item = &Note> {
        self.notes.iter().rev()
    }

    /// Writes `text` as the checkout's note, or removes the note when the
    /// cleaned text is empty. Returns whether anything changed.
    pub fn set(&mut self, checkout: Checkout, text: &str) -> bool {
        if !checkout.valid() {
            return false;
        }
        let text = clean(text);
        let position = self.notes.iter().position(|note| note.checkout == checkout);
        if position.is_some_and(|index| self.notes[index].text == text) {
            return false;
        }
        if let Some(index) = position {
            self.notes.remove(index);
        } else if text.is_empty() {
            return false;
        }
        if !text.is_empty() {
            self.notes.push(Note { checkout, text });
            if self.notes.len() > MAX_NOTES {
                let excess = self.notes.len() - MAX_NOTES;
                self.notes.drain(..excess);
            }
        }
        self.revision += 1;
        true
    }
}

#[cfg(test)]
mod tests;
