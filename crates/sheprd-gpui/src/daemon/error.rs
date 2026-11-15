//! Why starting or talking to the herdr server failed.

use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("daemon startup cancelled")]
    Cancelled,
    #[error("herdr server exited before accepting connections: {0}. Check the Herdr server log.")]
    Exited(std::process::ExitStatus),
    #[error("Could not start herdr server: {source}. Use Terminal > Reconnect to retry.")]
    Spawn {
        #[source]
        source: io::Error,
    },
    #[error("Timed out waiting for herdr server at {}. Check the Herdr server log and use Terminal > Reconnect.", .0.display())]
    Timeout(PathBuf),
    // keep-sorted end
}
