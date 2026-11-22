// ABOUTME: Owns one coalescing preference writer through drop and app shutdown.
// ABOUTME: Keeps pending writes discoverable until the worker finishes without UI joins.
use super::{Chrome, read_chrome};
use futures_channel::oneshot;
use spaceport_state_file::Writer;
use std::{
    io,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError, mpsc::SyncSender},
};

pub(super) struct Worker {
    completed: Mutex<Option<oneshot::Receiver<()>>>,
    writer: Mutex<Option<Writer<Chrome>>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;

impl Worker {
    pub(super) fn start(path: PathBuf, loaded: SyncSender<Chrome>) -> io::Result<Arc<Self>> {
        let (completion, completed) = oneshot::channel();
        let owner = Arc::new(Self {
            completed: Mutex::new(Some(completed)),
            writer: Mutex::new(None),
        });
        let lifetime = owner.clone();
        let writer = Writer::start_with_initialization("gpui-preferences", path, move |path| {
            let chrome = read_chrome(path).unwrap_or_else(|_| {
                tracing::warn!(
                    category = "preferences_read",
                    "Cannot read GPUI preferences"
                );
                Chrome::default()
            });
            let _ = loaded.send(chrome);
            // A quit observer can still find this owner after its window closes.
            // Closing the completion signal follows the last write and owner release.
            (lifetime, completion)
        })?;
        *owner.writer() = Some(writer);
        Ok(owner)
    }

    fn writer(&self) -> MutexGuard<'_, Option<Writer<Chrome>>> {
        self.writer.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn take_completion(&self) -> Option<oneshot::Receiver<()>> {
        self.completed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }

    pub(super) fn save(&self, chrome: Chrome) {
        if let Some(writer) = self.writer().as_ref() {
            writer.save(chrome);
        }
    }

    pub(super) fn stop(&self) {
        if let Some(writer) = self.writer().as_mut() {
            writer.stop();
        }
    }

    /// Takes the sole join handle. Call only off the UI thread.
    pub(super) fn finish(&self) {
        let writer = self.writer().take();
        if let Some(writer) = writer {
            writer.finish();
        }
    }
}
