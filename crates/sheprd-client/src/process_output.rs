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

/// How long output may still take to arrive once its child has exited: the
/// reader parks between polls, so its last read and send can trail the exit.
pub const DRAIN_GRACE: Duration = Duration::from_millis(250);

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

    /// Waits up to `grace` for the pipe to close, then stops the reader and
    /// returns what it kept. For diagnostics read after the child has exited,
    /// when a process it left behind may still hold a writer open.
    pub fn finish(mut self, grace: Duration) -> io::Result<Vec<u8>> {
        match self.output.recv_timeout(grace) {
            Ok(output) => return output,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
        self.stop_reader();
        self.output
            .try_recv()
            .unwrap_or_else(|_| Err(io::ErrorKind::BrokenPipe.into()))
    }

    fn stop_reader(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            if worker.join().is_err() {
                tracing::warn!("Child output reader panicked");
            }
        }
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
        self.stop_reader();
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
        // A stopped reader takes one last read of what is already waiting, so
        // bytes written before the stop are kept, and then returns; a writer
        // that never pauses cannot keep it running.
        let stopping = stop.load(Ordering::Acquire);
        if matches!(retain, Retain::Limited(_)) && output.len() == limit {
            return Ok(output);
        }
        let n = match read_pipe(pipe, &mut buffer) {
            Ok(0) => return Ok(output),
            Ok(n) => n,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock && stopping => {
                return Ok(output);
            }
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
        if stopping {
            return Ok(output);
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
