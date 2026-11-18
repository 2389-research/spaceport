// ABOUTME: Defines typed clipboard acquisition and destination failures.
// ABOUTME: Preserves native error sources for the user-facing localization boundary.
//! Clipboard failures shared by native readers and preparation.

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("A clipboard read is already in progress. Try pasting again shortly.")]
    Busy,
    #[error("The paste was cancelled because its destination changed. Try pasting again.")]
    Cancelled,
    #[error("The clipboard changed while reading. Try pasting again.")]
    Changed,
    #[error("Clipboard text is not valid UTF-8.")]
    Encoding(#[source] std::str::Utf8Error),
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    Native(#[from] super::macos::Error),
    #[error(
        "Could not read the clipboard; on Linux, install wl-clipboard for Wayland or xclip for X11."
    )]
    Process(#[source] io::Error),
    #[error("Clipboard content exceeds the {limit}-byte limit.")]
    Size { limit: usize },
    #[error("Clipboard acquisition timed out.")]
    Timeout,
    #[error("Remote clipboard acquisition is not supported on this platform.")]
    Unsupported,
    // keep-sorted end
}
