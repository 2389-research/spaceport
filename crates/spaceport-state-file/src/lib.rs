// ABOUTME: Reads bounded state files and writes atomic snapshots on a worker.
// ABOUTME: Preserves rejected source bytes in a private sibling file.
//! Small app-owned JSON state files. Reads are bounded, writes replace the
//! file atomically, and the latest value is saved on a worker so a change made
//! on the UI thread never waits for the disk.

mod error;
mod localize;

pub use error::{Error, Result};

use serde::Serialize;
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

/// The file's bytes, or `None` when it does not exist yet.
pub fn read(path: &Path, limit: u64) -> Result<Option<Vec<u8>>> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(Error::Size {
            path: path.to_owned(),
            limit,
        });
    }
    Ok(Some(bytes))
}

/// Keeps the bytes a caller rejected before it allows replacement writes to `path`.
pub fn preserve_rejected(path: &Path, bytes: &[u8]) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::MissingRoot(path.to_owned()))?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let mut prefix = path
        .file_name()
        .ok_or_else(|| Error::MissingRoot(path.to_owned()))?
        .to_owned();
    prefix.push(".rejected-");
    let mut backup = tempfile::Builder::new()
        .prefix(&prefix)
        .tempfile_in(parent)?;
    backup.write_all(bytes)?;
    backup.as_file().sync_all()?;
    #[cfg(unix)]
    fs::File::open(parent)?.sync_all()?;
    let (_, saved) = backup.keep().map_err(|error| error.error)?;
    tracing::warn!(path = %saved.display(), "Preserved rejected state file");
    Ok(saved)
}

pub fn write<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::MissingRoot(path.to_owned()))?;
    fs::create_dir_all(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    // Buffered: the serializer writes each escape and punctuation mark on its
    // own, which unbuffered is a system call apiece.
    let mut buffer = io::BufWriter::new(&mut file);
    serde_json::to_writer(&mut buffer, value)?;
    buffer.write_all(b"\n")?;
    buffer.flush()?;
    drop(buffer);
    // No fsync: on macOS it is F_FULLFSYNC, which can outlast GPUI's 100 ms
    // quit budget. The rename still replaces the file atomically, and a file
    // torn by power loss is rejected on load in favor of the defaults.
    file.persist(path).map_err(|error| error.error)?;
    Ok(())
}

/// Saves the most recent value handed to it. Values replaced before the
/// worker reads them are never written.
pub struct Writer<T> {
    pending: Arc<Mutex<Option<T>>>,
    wake: Option<mpsc::SyncSender<()>>,
    worker: JoinHandle<()>,
}

impl<T: Serialize + Send + 'static> Writer<T> {
    pub fn start(name: &str, path: PathBuf) -> io::Result<Self> {
        Self::start_with_initialization(name, path, |_| ())
    }

    /// Runs `initialize` on the worker before any write, and keeps what it
    /// returns until the worker exits, so a guard outlives the last write.
    pub fn start_with_initialization<G: Send + 'static>(
        name: &str,
        path: PathBuf,
        initialize: impl FnOnce(&Path) -> G + Send + 'static,
    ) -> io::Result<Self> {
        let pending = Arc::new(Mutex::new(None::<T>));
        let mailbox = pending.clone();
        let (wake, receiver) = mpsc::sync_channel(1);
        let label = name.to_owned();
        let worker = thread::Builder::new().name(name.into()).spawn(move || {
            let _lifetime = initialize(&path);
            for () in receiver {
                let snapshot = mailbox.lock().ok().and_then(|mut pending| pending.take());
                if let Some(snapshot) = snapshot
                    && let Err(error) = write(&path, &snapshot)
                {
                    tracing::warn!(%error, file = label, "Cannot save state");
                }
            }
        })?;
        Ok(Self {
            pending,
            wake: Some(wake),
            worker,
        })
    }

    /// Replaces the pending value. Does nothing once the writer is stopped.
    pub fn save(&self, value: T) {
        let Some(wake) = &self.wake else {
            return;
        };
        if let Ok(mut pending) = self.pending.lock() {
            *pending = Some(value);
        }
        // A full wake queue already guarantees that the latest value is read.
        let _ = wake.try_send(());
    }

    /// Closes the wake sender without waiting. The worker still writes the
    /// value already pending, then exits.
    pub fn stop(&mut self) {
        drop(self.wake.take());
    }

    /// Writes whatever is still pending, then stops the worker. Blocks, so
    /// callers run it off the UI thread.
    pub fn finish(mut self) {
        self.stop();
        if self.worker.join().is_err() {
            tracing::warn!("State writer panicked");
        }
    }
}

#[cfg(test)]
mod tests;
