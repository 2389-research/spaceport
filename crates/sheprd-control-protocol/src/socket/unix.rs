// ABOUTME: Owns a private Unix control socket and its recovery lock.
// ABOUTME: Removes only the bound socket while retaining exclusive address ownership.
//! The control channel on Unix: a socket file in the user's private state
//! directory, readable by the user alone.
use std::{
    fs::{File, OpenOptions, TryLockError},
    io::{self, Read, Write},
    os::unix::{
        fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

pub(super) fn address(state: &Path) -> crate::Result<PathBuf> {
    Ok(state.join("control.sock"))
}

struct Binding {
    path: PathBuf,
    device: u64,
    inode: u64,
    _lock: File,
}

#[derive(Clone)]
pub(super) struct Cleanup(Arc<Binding>);

pub(super) struct Listener {
    listener: UnixListener,
    cleanup: Cleanup,
}

impl Listener {
    /// Binds `path`, replacing a socket no process answers on. Another live
    /// server keeps its socket and this one does not start.
    pub(super) fn bind(path: &Path) -> crate::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
        // Keep this stable file after shutdown. Unlinking it would let two
        // starters lock different inodes for the same socket address.
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .open(path.with_extension("lock"))?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                return Err(crate::Error::SocketInUse {
                    path: path.to_owned(),
                });
            }
            Err(TryLockError::Error(error)) => return Err(error.into()),
        }
        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
                if UnixStream::connect(path).is_ok() {
                    return Err(crate::Error::SocketInUse {
                        path: path.to_owned(),
                    });
                }
                std::fs::remove_file(path)?;
                UnixListener::bind(path)?
            }
            Err(error) => return Err(error.into()),
        };
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        let metadata = std::fs::symlink_metadata(path)?;
        let cleanup = Cleanup(Arc::new(Binding {
            path: path.to_owned(),
            device: metadata.dev(),
            inode: metadata.ino(),
            _lock: lock,
        }));
        Ok(Self { listener, cleanup })
    }

    pub(super) fn accept(&self) -> io::Result<Connection> {
        Ok(Connection(self.listener.accept()?.0))
    }

    pub(super) fn cleanup(&self) -> Cleanup {
        self.cleanup.clone()
    }
}

/// Removes the exact bound socket. The live lease excludes other starters.
pub(super) fn release(cleanup: &Cleanup) {
    let binding = &cleanup.0;
    if std::fs::symlink_metadata(&binding.path).is_ok_and(|metadata| {
        metadata.file_type().is_socket()
            && metadata.dev() == binding.device
            && metadata.ino() == binding.inode
    }) {
        let _ = std::fs::remove_file(&binding.path);
    }
}

pub(super) struct Connection(UnixStream);

impl Connection {
    pub(super) fn prepare(&mut self, timeout: Duration) -> io::Result<()> {
        self.0.set_read_timeout(Some(timeout))?;
        self.0.set_write_timeout(Some(timeout))
    }

    /// Switches to nonblocking reads while the UI answers, so
    /// [`Self::peer_closed`] can look without waiting.
    pub(super) fn watch(&self, watching: bool) -> io::Result<()> {
        self.0.set_nonblocking(watching)
    }

    /// A request is one line only. EOF, another byte, or an I/O error ends
    /// it; WouldBlock means the peer is still waiting.
    pub(super) fn peer_closed(&self) -> bool {
        !matches!((&self.0).read(&mut [0]), Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted))
    }
}

impl Read for &Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.0).read(buf)
    }
}

impl Write for &Connection {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.0).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.0).flush()
    }
}

/// Connects to the server at `path`; `io_timeout` bounds sending.
pub(super) fn connect(
    path: &Path,
    io_timeout: Duration,
    read_timeout: Duration,
) -> crate::Result<Connection> {
    let stream = UnixStream::connect(path).map_err(|source| crate::Error::Unavailable {
        path: path.to_owned(),
        source,
    })?;
    stream.set_write_timeout(Some(io_timeout))?;
    stream.set_read_timeout(Some(read_timeout))?;
    Ok(Connection(stream))
}
