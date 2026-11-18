// ABOUTME: Reads Linux clipboard formats through bounded native helper processes.
// ABOUTME: Keeps timeout, output limits, and exact child cleanup together.
use super::*;
use std::{
    io::{self, Read},
    os::{fd::OwnedFd, unix::net::UnixStream},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(target_os = "linux")]
pub(super) fn read(mode: Mode) -> Result<Option<ClipboardItem>> {
    read_commands(
        mode,
        wayland_session(),
        std::env::var_os("DISPLAY").is_some(),
        |program, args, limit, deadline| {
            output(
                sheprd_client::new_command(program).args(args),
                limit,
                deadline,
            )
        },
    )
}

/// Whether a Wayland compositor can actually answer, decided by its socket
/// rather than by the variable alone. A session started from a display
/// manager can inherit `WAYLAND_DISPLAY` while running through X11; the
/// variable then names a socket nothing listens on, `wl-paste` fails, and
/// the X11 selection is never read because XWayland must not be a fallback
/// for a live Wayland session.
#[cfg(target_os = "linux")]
fn wayland_session() -> bool {
    wayland_socket(
        std::env::var_os("WAYLAND_DISPLAY").as_deref(),
        std::env::var_os("XDG_RUNTIME_DIR").as_deref(),
    )
}

/// The compositor socket named by `WAYLAND_DISPLAY`, resolved against
/// `XDG_RUNTIME_DIR` when the name is relative, must accept a connection.
/// A crashed compositor can leave its socket on disk, so the file existing
/// is not enough: connecting proves that something still answers.
///
/// The attempt never blocks. A compositor that is busy (its accept queue is
/// full) still exists, so it keeps the Wayland helper; only a socket that
/// refuses the connection falls back to X11, which is why this runs before
/// the paste deadline rather than inside it.
#[cfg(target_os = "linux")]
fn wayland_socket(display: Option<&std::ffi::OsStr>, runtime: Option<&std::ffi::OsStr>) -> bool {
    use nix::sys::socket::{AddressFamily, SockFlag, SockType, UnixAddr, connect, socket};
    use rustix::event::{PollFd, PollFlags, Timespec, poll};
    use std::os::fd::AsRawFd;

    let Some(display) = display else {
        return false;
    };
    let display = std::path::Path::new(display);
    let socket_path = if display.is_absolute() {
        display.to_path_buf()
    } else {
        let Some(runtime) = runtime else {
            return false;
        };
        std::path::Path::new(runtime).join(display)
    };
    let Ok(address) = UnixAddr::new(&socket_path) else {
        return false;
    };
    let flags = SockFlag::SOCK_NONBLOCK | SockFlag::SOCK_CLOEXEC;
    let Ok(socket) = socket(AddressFamily::Unix, SockType::Stream, flags, None) else {
        return false;
    };
    match connect(socket.as_raw_fd(), &address) {
        Ok(()) => true,
        // A full accept queue means a compositor is there but busy.
        Err(nix::errno::Errno::EAGAIN) => true,
        Err(nix::errno::Errno::EINPROGRESS) => {
            let mut fds = [PollFd::new(&socket, PollFlags::OUT)];
            let timeout = Timespec {
                tv_sec: 0,
                tv_nsec: 50_000_000,
            };
            match poll(&mut fds, Some(&timeout)) {
                // Still pending after the budget: assume a busy compositor
                // rather than reading XWayland behind a live Wayland session.
                Ok(0) | Err(_) => true,
                Ok(_) => !fds[0]
                    .revents()
                    .intersects(PollFlags::ERR | PollFlags::HUP | PollFlags::NVAL),
            }
        }
        Err(_) => false,
    }
}

fn read_commands(
    mode: Mode,
    wayland: bool,
    x11: bool,
    mut run: impl FnMut(&str, &[&str], usize, Instant) -> Result<Option<Vec<u8>>>,
) -> Result<Option<ClipboardItem>> {
    // One deadline for the entire acquisition, not a fresh timeout per format.
    let deadline = Instant::now() + Duration::from_secs(3);
    read_with(mode, |format, limit| {
        let types: &[&str] = match format {
            None => &["text/plain;charset=utf-8", "text/plain", "UTF8_STRING"],
            Some(ImageFormat::Jpeg) => &["image/jpeg", "image/jpg"],
            Some(format) => &[format.mime_type()],
        };
        for mime in types {
            for (enabled, program, args) in [
                (
                    wayland && *mime != "UTF8_STRING",
                    "wl-paste",
                    &["--no-newline", "--type", mime][..],
                ),
                (
                    // XWayland may hold a different selection. Never use
                    // it as a fallback for an empty/unsupported Wayland offer.
                    x11 && !wayland,
                    "xclip",
                    &["-selection", "clipboard", "-t", mime, "-o"][..],
                ),
            ] {
                if !enabled {
                    continue;
                }
                if let Some(bytes) = run(program, args, limit, deadline)? {
                    if let Some(format) = format {
                        // Some X11 owners return text even for an image target.
                        // Inspect only the signature, never decode untrusted pixels.
                        let expected = match format {
                            ImageFormat::Png => image::ImageFormat::Png,
                            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
                            ImageFormat::Gif => image::ImageFormat::Gif,
                            ImageFormat::Webp => image::ImageFormat::WebP,
                            ImageFormat::Bmp => image::ImageFormat::Bmp,
                            ImageFormat::Tiff => image::ImageFormat::Tiff,
                            ImageFormat::Svg | ImageFormat::Ico | ImageFormat::Pnm => {
                                return Err(crate::Error::ImageFormat);
                            }
                        };
                        if image::guess_format(&bytes).ok() != Some(expected) {
                            continue;
                        }
                    }
                    return Ok(Some(bytes));
                }
            }
        }
        Ok(None)
    })
}

fn output(command: &mut Command, limit: usize, deadline: Instant) -> Result<Option<Vec<u8>>> {
    if Instant::now() >= deadline {
        return Err(Error::Timeout.into());
    }
    // A short read timeout avoids reader threads that can outlive a
    // timed-out child when a descendant inherits stdout. It blocks until
    // output arrives, where sleeping between reads of a small socket
    // buffer paced a large image past the deadline. Set it before the
    // child can exit: macOS refuses socket options once the peer closed.
    // Never merge stderr into images.
    let (mut reader, writer) = UnixStream::pair().map_err(Error::Process)?;
    reader
        .set_read_timeout(Some(Duration::from_millis(50)))
        .map_err(Error::Process)?;
    let spawned = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(OwnedFd::from(writer)))
        .stderr(Stdio::null())
        .spawn();
    command.stdout(Stdio::null());
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            return Err(Error::Process(error).into());
        }
    };
    let result: std::result::Result<_, Error> = (|| {
        let mut bytes = Vec::new();
        let mut buffer = vec![0; 64 * 1024];
        let mut eof = false;
        loop {
            if Instant::now() >= deadline {
                return Err(Error::Timeout);
            }
            if !eof {
                match reader.read(&mut buffer) {
                    Ok(0) => eof = true,
                    Ok(count) => {
                        if count > limit - bytes.len() {
                            return Err(Error::Size { limit });
                        }
                        bytes.extend_from_slice(&buffer[..count]);
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock
                                | io::ErrorKind::TimedOut
                                | io::ErrorKind::Interrupted
                        ) => {}
                    Err(error) => {
                        return Err(Error::Process(error));
                    }
                }
                continue;
            }
            if let Some(status) = child.try_wait().map_err(Error::Process)? {
                return Ok(status.success().then_some(bytes));
            }
            thread::sleep(Duration::from_millis(5));
        }
    })();
    if result.is_err() {
        let _ = child.kill();
    }
    let waited = child.wait().map_err(Error::Process);
    let result = result?;
    waited?;
    Ok(result)
}

#[cfg(test)]
mod tests;
