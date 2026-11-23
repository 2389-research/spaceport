//! The app's copy of the notes the user keeps on checkouts, and the file it is
//! saved in. The notes themselves — their bounds, their one safe line of text,
//! and the order they are kept in — are `spaceport-sidebar`'s
//! [`worktree_notes`](spaceport_sidebar::worktree_notes); this holds them where
//! every window reads them and saves them on a worker.
use gpui::{App, Global};
use spaceport_sidebar::worktree_notes as model;
use std::path::PathBuf;

pub(crate) use model::{Checkout, Note, clean};

const FILE: &str = "worktree-notes.json";

/// The app's notes, oldest edit first.
#[derive(Default)]
pub(crate) struct Notes {
    notes: model::Notes,
    writer: Option<spaceport_state_file::Writer<model::Saved>>,
    quitting: bool,
}

impl Global for Notes {}

impl Notes {
    fn path() -> Option<PathBuf> {
        crate::preferences::state_dir().map(|dir| dir.join(FILE))
    }

    /// Called before starting GPUI, like the browser tabs. A missing or
    /// damaged file starts with no notes.
    pub(crate) fn load() -> Self {
        Self::at(Self::path())
    }

    fn at(path: Option<PathBuf>) -> Self {
        let notes = path
            .as_deref()
            .map(|path| {
                spaceport_state_file::read(path, model::MAX_FILE_BYTES)
                    .map_err(crate::Error::from)
                    .and_then(|bytes| bytes.as_deref().map_or(Ok(model::Notes::default()), parse))
            })
            .transpose()
            .unwrap_or_else(|error| {
                tracing::warn!(%error, "Cannot restore worktree notes");
                None
            })
            .unwrap_or_default();
        let writer = path.and_then(|path| {
            match spaceport_state_file::Writer::start("worktree-notes", path) {
                Ok(writer) => Some(writer),
                Err(error) => {
                    tracing::warn!(%error, "Cannot start the worktree notes worker");
                    None
                }
            }
        });
        Self {
            notes,
            writer,
            quitting: false,
        }
    }

    pub(crate) fn install(self, cx: &mut App) {
        cx.set_global(self);
        cx.on_app_quit(|cx| {
            let notes = cx.global_mut::<Self>();
            notes.quitting = true;
            let writer = notes.writer.take();
            cx.background_executor().spawn(async move {
                if let Some(writer) = writer {
                    writer.finish();
                }
            })
        })
        .detach();
    }

    /// Runs `f` against the app's notes, creating an unsaved set for
    /// fixtures that never installed them.
    pub(crate) fn update<R>(cx: &mut App, f: impl FnOnce(&mut Self) -> R) -> R {
        if !cx.has_global::<Self>() {
            cx.set_global(Self::default());
        }
        f(cx.global_mut::<Self>())
    }

    /// The app's notes, or none for fixtures that never installed them.
    pub(crate) fn of(cx: &App) -> Option<&Self> {
        cx.try_global::<Self>()
    }

    pub(crate) fn get(&self, endpoint: &str, repo_key: &str, branch: &str) -> Option<&str> {
        self.notes.get(endpoint, repo_key, branch)
    }

    pub(crate) fn revision(&self) -> u64 {
        self.notes.revision()
    }

    /// Every note, most recently edited first.
    pub(crate) fn recent(&self) -> impl Iterator<Item = &Note> {
        self.notes.recent()
    }

    /// Writes `text` as the checkout's note, or removes the note when the
    /// cleaned text is empty. Returns whether anything changed.
    pub(crate) fn set(&mut self, checkout: Checkout, text: &str) -> bool {
        if !self.notes.set(checkout, text) {
            return false;
        }
        self.save();
        true
    }

    fn save(&self) {
        if self.quitting {
            return;
        }
        if let Some(writer) = &self.writer {
            writer.save(self.notes.saved());
        }
    }
}

/// The notes `bytes` holds, or the reason a file of them cannot be read.
fn parse(bytes: &[u8]) -> crate::Result<model::Notes> {
    let saved: model::Saved = serde_json::from_slice(bytes)?;
    model::Notes::from_saved(saved.notes).ok_or(crate::Error::InvalidWorktreeNotes)
}

#[cfg(test)]
mod tests;
