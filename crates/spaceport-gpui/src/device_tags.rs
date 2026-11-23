//! The app's copy of every device's local tags, and the worker that saves it.
//! The tags themselves, their rules, and the file they are saved in are
//! `spaceport-sidebar`'s [`device_tags`](spaceport_sidebar::device_tags); this
//! holds the copy every window reads and decides when it is written.
use spaceport_sidebar::device_tags::{FILE, read, write};
pub(crate) use spaceport_sidebar::device_tags::{Tags, normalized, offered, same};

use gpui::{App, BorrowAppContext, Global};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
    time::{SystemTime, UNIX_EPOCH},
};

/// This app's local tags.
#[derive(Default)]
pub(crate) struct DeviceTags {
    tags: Arc<Tags>,
    file: Option<PathBuf>,
    /// Whether the saved copy was read: saving before then could overwrite
    /// tags left by another window from an earlier app run.
    loaded: bool,
    /// The newest copy to write, shared with the writer so a slow write
    /// never puts an older copy back. Only ever held for a moment, as the UI
    /// thread takes it.
    saving: Arc<Mutex<Option<Arc<Tags>>>>,
    /// Held by the writer across a write, so they land one at a time.
    writing: Arc<Mutex<()>>,
}

impl Global for DeviceTags {}

impl DeviceTags {
    /// Reads the saved copy off the UI thread.
    pub(crate) fn install(cx: &mut App) {
        let file = crate::preferences::state_dir().map(|dir| dir.join(FILE));
        cx.set_global(Self {
            file: file.clone(),
            ..Self::default()
        });
        let read = cx
            .background_executor()
            .spawn(async move { file.as_deref().map(read).unwrap_or_default() });
        cx.spawn(async move |cx| {
            let tags = read.await;
            cx.update(|cx| Self::finish_loading(cx, tags));
        })
        .detach();
    }

    /// Takes the saved copy, keeping any edit made while it was being read.
    fn finish_loading(cx: &mut App, saved: Tags) {
        let edited = cx.update_global::<Self, _>(|this, _| {
            let edited = !this.tags.is_empty();
            this.tags = Arc::new(saved.with_edits(&this.tags));
            this.loaded = true;
            edited
        });
        if edited {
            Self::save(cx);
        }
    }

    /// The copy to draw from; empty before [`Self::install`], as in tests.
    pub(crate) fn current(cx: &App) -> Arc<Tags> {
        cx.try_global::<Self>()
            .map(|this| this.tags.clone())
            .unwrap_or_default()
    }

    /// Replaces a device's local tags and saves them.
    pub(crate) fn set(cx: &mut App, id: &str, tags: Vec<String>) {
        if !cx.has_global::<Self>() {
            return;
        }
        let changed = cx.update_global::<Self, _>(|this, _| {
            let mut copy = (*this.tags).clone();
            if !copy.edit(id, tags, now_ms()) {
                return false;
            }
            this.tags = Arc::new(copy);
            true
        });
        if changed {
            Self::save(cx);
        }
    }

    fn save(cx: &mut App) {
        let this = cx.global::<Self>();
        let Some(file) = this.file.clone().filter(|_| this.loaded) else {
            return;
        };
        let (saving, writing) = (this.saving.clone(), this.writing.clone());
        *saving.lock().unwrap_or_else(PoisonError::into_inner) = Some(this.tags.clone());
        cx.background_executor()
            .spawn(async move {
                // The newest copy is taken with the write's turn in hand, so
                // the last write holds it.
                let _writing = writing.lock().unwrap_or_else(PoisonError::into_inner);
                let newest = saving.lock().unwrap_or_else(PoisonError::into_inner).take();
                if let Some(tags) = newest
                    && let Err(error) = write(&file, &tags)
                {
                    tracing::warn!(%error, "Could not save device tags");
                }
            })
            .detach();
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| {
            u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
mod tests;
