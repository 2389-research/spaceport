// ABOUTME: Serves byte-mode lifecycle peers on native named pipes.
// ABOUTME: Uses nonblocking handles and bounded peeks without pipe-drop flush waits.
use super::TIMEOUT;
use interprocess::os::windows::named_pipe::{PipeListener, PipeListenerOptions, pipe_mode};
use std::{
    fs::File,
    io::{self, Read, Write},
    os::windows::io::{AsHandle, OwnedHandle},
    path::Path,
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

pub(super) struct Listener(PipeListener<pipe_mode::Bytes, pipe_mode::Bytes>);

impl Listener {
    pub(super) fn bind(path: &Path) -> io::Result<Self> {
        // Match Herdr's daemon path mapping, rather than the control socket's name.
        let address = format!(r"\\.\pipe\{}", path.to_string_lossy());
        Ok(Self(
            PipeListenerOptions::new()
                .path(address)
                .accept_remote(false)
                .nonblocking(true)
                .create_duplex::<pipe_mode::Bytes>()?,
        ))
    }

    pub(super) fn accept(&self) -> io::Result<Stream> {
        let until = Instant::now() + TIMEOUT;
        let stream = loop {
            match self.0.accept() {
                Ok(stream) => break stream,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => pause(until)?,
                Err(error) => return Err(error),
            }
        };
        stream.set_nonblocking(true)?;
        // File drop closes this exact handle without interprocess's flush worker.
        let handle: OwnedHandle = stream.try_into().map_err(|_| io::ErrorKind::InvalidData)?;
        Ok(Stream {
            file: Mutex::new(Some(handle.into())),
            read_timeout: TIMEOUT,
        })
    }
}

pub(in crate::endpoint::lifecycle_tests) struct Stream {
    file: Mutex<Option<File>>,
    read_timeout: Duration,
}

impl Stream {
    pub(in crate::endpoint::lifecycle_tests) fn shutdown(
        &self,
        _: std::net::Shutdown,
    ) -> io::Result<()> {
        self.file.lock().unwrap().take();
        Ok(())
    }

    pub(in crate::endpoint::lifecycle_tests) fn set_read_timeout(
        &mut self,
        timeout: Option<Duration>,
    ) -> io::Result<()> {
        // Every fixture read stays bounded, including probes for an empty queue.
        self.read_timeout = timeout
            .filter(|value| !value.is_zero())
            .ok_or(io::ErrorKind::InvalidInput)?;
        Ok(())
    }
}

fn pause(until: Instant) -> io::Result<()> {
    let left = until.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Err(io::ErrorKind::TimedOut.into());
    }
    thread::sleep(left.min(Duration::from_millis(1)));
    Ok(())
}

impl Read for Stream {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let until = Instant::now() + self.read_timeout;
        loop {
            {
                let mut file = self.file.lock().unwrap();
                let Some(file) = file.as_mut() else {
                    return Ok(0);
                };
                match sheprd_client::pipe_available(file.as_handle())? {
                    None => return Ok(0),
                    Some(0) => {}
                    Some(available) => {
                        let count = bytes.len().min(available as usize);
                        return file.read(&mut bytes[..count]);
                    }
                }
            }
            pause(until)?;
        }
    }
}

impl Write for Stream {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let until = Instant::now() + TIMEOUT;
        loop {
            {
                let mut file = self.file.lock().unwrap();
                let file = file.as_mut().ok_or(io::ErrorKind::BrokenPipe)?;
                match file.write(bytes) {
                    Ok(0) => {}
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    result => return result,
                }
            }
            pause(until)?;
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        // Writes are synchronous, nonblocking byte-mode writes; no flush is needed.
        Ok(())
    }
}
