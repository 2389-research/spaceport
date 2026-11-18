// ABOUTME: Captures bounded child-pipe output on cancellable background readers.
// ABOUTME: Closes each pipe and joins its reader without waiting for inherited writers.
use std::{
    fs::File,
    io::{self, Read},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

#[cfg(unix)]
pub type PipeHandle = std::os::fd::OwnedFd;
#[cfg(windows)]
pub type PipeHandle = std::os::windows::io::OwnedHandle;

const POLL: Duration = Duration::from_millis(25);

/// Which bytes to keep while reading a pipe.
#[derive(Clone, Copy, Debug)]
pub enum Retain {
    /// Stop reading after this many bytes, including an overflow sentinel when needed.
    Limited(u64),
    /// Keep this prefix and drain the remaining output.
    Prefix(u64),
    /// Keep only the most recent bytes until EOF.
    Tail(u64),
}

/// One owned pipe reader. Create and drop it only off the UI thread.
#[derive(Debug)]
pub struct Capture {
    output: mpsc::Receiver<io::Result<Vec<u8>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Capture {
    pub fn recv_timeout(
        &self,
        timeout: Duration,
    ) -> Result<io::Result<Vec<u8>>, mpsc::RecvTimeoutError> {
        self.output.recv_timeout(timeout)
    }
}

/// Captures an exclusively owned pipe without blocking indefinitely on reads.
pub fn capture(pipe: impl Into<PipeHandle>, retain: Retain) -> io::Result<Capture> {
    let mut pipe = File::from(pipe.into());
    configure(&pipe)?;
    let limit = match retain {
        Retain::Limited(n) | Retain::Prefix(n) | Retain::Tail(n) => n,
    };
    let limit = usize::try_from(limit)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let (sent, output) = mpsc::sync_channel(1);
    let stop = Arc::new(AtomicBool::new(false));
    let stopped = stop.clone();
    let worker = thread::Builder::new()
        .name("spaceport-command-output".into())
        .spawn(move || {
            let result = collect(&mut pipe, retain, limit, &stopped);
            let _ = sent.send(result);
        })?;
    Ok(Capture {
        output,
        stop,
        worker: Some(worker),
    })
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            if worker.join().is_err() {
                tracing::warn!("Child output reader panicked");
            }
        }
    }
}

fn collect(
    pipe: &mut File,
    retain: Retain,
    limit: usize,
    stop: &AtomicBool,
) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        if stop.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if matches!(retain, Retain::Limited(_)) && output.len() == limit {
            return Ok(output);
        }
        let n = match read_pipe(pipe, &mut buffer) {
            Ok(0) => return Ok(output),
            Ok(n) => n,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::park_timeout(POLL);
                continue;
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        match retain {
            Retain::Limited(_) | Retain::Prefix(_) => {
                output.extend_from_slice(&buffer[..n.min(limit.saturating_sub(output.len()))]);
            }
            Retain::Tail(_) if n >= limit => {
                output.clear();
                output.extend_from_slice(&buffer[n - limit..n]);
            }
            Retain::Tail(_) => {
                let discard = output.len().saturating_add(n).saturating_sub(limit);
                output.drain(..discard);
                output.extend_from_slice(&buffer[..n]);
            }
        }
    }
}

#[cfg(unix)]
fn configure(pipe: &File) -> io::Result<()> {
    use rustix::fs::{FileType, OFlags, fcntl_getfl, fcntl_setfl, fstat};
    if FileType::from_raw_mode(fstat(pipe)?.st_mode) != FileType::Fifo {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    fcntl_setfl(pipe, fcntl_getfl(pipe)? | OFlags::NONBLOCK)?;
    Ok(())
}

#[cfg(windows)]
fn configure(pipe: &File) -> io::Result<()> {
    use std::os::windows::io::AsHandle;
    crate::transport::pipe_available(pipe.as_handle())?;
    Ok(())
}

#[cfg(unix)]
fn read_pipe(pipe: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    pipe.read(buffer)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests;

#[cfg(windows)]
fn read_pipe(pipe: &mut File, buffer: &mut [u8]) -> io::Result<usize> {
    use std::os::windows::io::AsHandle;
    match crate::transport::pipe_available(pipe.as_handle())? {
        None => Ok(0),
        Some(0) => Err(io::ErrorKind::WouldBlock.into()),
        Some(available) => {
            let take = (available as usize).min(buffer.len());
            pipe.read(&mut buffer[..take])
        }
    }
}
