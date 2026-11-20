// ABOUTME: Serves the bounded profile observer through a private Windows named pipe.
// ABOUTME: Reuses client SID checks and the control channel's whole-reply buffer policy.
use super::report::MAX_LINE;
use interprocess::os::windows::{
    ToWtf16 as _,
    named_pipe::{DuplexPipeStream, PipeListener, PipeListenerOptions, pipe_mode},
    security_descriptor::SecurityDescriptor,
};
use std::{
    io::{self, Read, Write},
    os::windows::io::AsHandle as _,
    path::Path,
    thread,
    time::{Duration, Instant},
};

type Pipe = DuplexPipeStream<pipe_mode::Bytes>;

pub(super) struct Listener(PipeListener<pipe_mode::Bytes, pipe_mode::Bytes>);

impl Listener {
    pub(super) fn bind(path: &Path) -> io::Result<Self> {
        let sid = sheprd_client::private_endpoint::user_sid()?;
        let sddl = format!("O:{sid}D:P(A;;GA;;;{sid})");
        let descriptor =
            SecurityDescriptor::deserialize(&sddl.to_wtf_16().map_err(io::Error::other)?)?;
        PipeListenerOptions::new()
            .path(path)
            .accept_remote(false)
            .security_descriptor(Some(descriptor))
            .output_buffer_size_hint((MAX_LINE + 1) as u32)
            .create_duplex::<pipe_mode::Bytes>()
            .map(Self)
    }

    pub(super) fn accept(&self) -> io::Result<Stream> {
        Ok(Stream {
            pipe: self.0.accept()?,
            timeout: Duration::from_secs(15),
        })
    }
}

pub(super) struct Stream {
    pipe: Pipe,
    timeout: Duration,
}

impl Stream {
    pub(super) fn set_read_timeout(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        self.timeout = timeout
            .filter(|timeout| !timeout.is_zero())
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "profile read needs a deadline")
            })?;
        Ok(())
    }
}

impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let deadline = Instant::now() + self.timeout;
        let mut retry = Duration::from_millis(1);
        loop {
            match sheprd_client::pipe_available(self.pipe.as_handle())? {
                None => return Ok(0),
                Some(0) => {}
                Some(available) => {
                    let count = buffer.len().min(available as usize);
                    return (&self.pipe).read(&mut buffer[..count]);
                }
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(io::ErrorKind::TimedOut.into());
            }
            thread::sleep(retry.min(remaining));
            retry = (retry * 2).min(Duration::from_millis(25));
        }
    }
}

impl Write for Stream {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // The buffer holds the complete bounded answer, including its newline.
        (&self.pipe).write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        // A pipe flush waits for the peer. Written bytes already reside in the buffer.
        Ok(())
    }
}
