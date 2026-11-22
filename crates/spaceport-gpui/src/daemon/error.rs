//! Why starting or talking to the Herdr daemon failed.

use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("Daemon startup cancelled.")]
    Cancelled,
    #[error("The Herdr daemon exited before accepting connections: {0}. Check the daemon's log.")]
    Exited(std::process::ExitStatus),
    #[error("Could not start the Herdr daemon: {source}. Use Terminal > Reconnect to retry.")]
    Spawn {
        #[source]
        source: io::Error,
    },
    #[error("Timed out waiting for the Herdr daemon at {}. Check the daemon's log and use Terminal > Reconnect.", .0.display())]
    Timeout(PathBuf),
    // keep-sorted end
}
