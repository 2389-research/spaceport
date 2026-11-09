//! Git failures from read-only checkout probes and close checks.

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("git {operation} failed: {details}")]
    Failed {
        operation: &'static str,
        details: String,
    },
    #[error("Could not {operation}.")]
    Process {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("Git worker stopped. Retry the operation.")]
    Worker,
    // keep-sorted end
}
