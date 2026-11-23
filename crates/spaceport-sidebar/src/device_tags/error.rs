//! Why local device tags could not be saved.

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // keep-sorted start
    #[error("Could not save device tags to {}", path.display())]
    Save {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    // keep-sorted end
}
