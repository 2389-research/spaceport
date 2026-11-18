// ABOUTME: Serves isolated lifecycle fixtures through each platform's real transport.
// ABOUTME: Bounds fixture I/O and closes only the accepted peer on shutdown.
use std::{io, path::Path, time::Duration};

#[cfg(unix)]
pub(super) use std::os::unix::net::UnixStream as Stream;
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(super) use windows::Stream;

pub(super) const TIMEOUT: Duration = Duration::from_secs(3);

pub(super) struct Listener {
    #[cfg(unix)]
    inner: std::os::unix::net::UnixListener,
    #[cfg(windows)]
    inner: windows::Listener,
}

impl Listener {
    pub(super) fn bind(path: &Path) -> io::Result<Self> {
        #[cfg(unix)]
        let inner = std::os::unix::net::UnixListener::bind(path)?;
        #[cfg(windows)]
        let inner = windows::Listener::bind(path)?;
        Ok(Self { inner })
    }

    pub(super) fn accept(&self) -> io::Result<Stream> {
        #[cfg(unix)]
        {
            let (stream, _) = self.inner.accept()?;
            stream.set_nonblocking(false)?;
            stream.set_read_timeout(Some(TIMEOUT))?;
            stream.set_write_timeout(Some(TIMEOUT))?;
            Ok(stream)
        }
        #[cfg(windows)]
        self.inner.accept()
    }
}
