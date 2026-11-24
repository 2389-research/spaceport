//! A failed client storage step, comparable so a retried operation can warn
//! once per distinct failure rather than once per attempt. The app words and
//! places the warning; the failure itself is a value here, because the
//! catalog's two-second retry decides which failures it has already named.

use spaceport_client::StorageOperation;
use std::{error::Error as StdError, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageFailure {
    operation: StorageOperation,
    path: PathBuf,
    /// The redacted display text, which tells apart causes on the same path.
    message: String,
}

impl StorageFailure {
    /// The outermost client storage step anywhere in `error`'s source chain.
    pub fn find(error: &(dyn StdError + 'static)) -> Option<Self> {
        std::iter::successors(Some(error), |&error| error.source()).find_map(|error| {
            match error.downcast_ref::<spaceport_client::Error>()? {
                storage @ spaceport_client::Error::Storage {
                    operation, path, ..
                } => Some(Self {
                    operation: operation.clone(),
                    path: path.clone(),
                    message: storage.to_string(),
                }),
                _ => None,
            }
        })
    }

    /// Log the failed step with its operation and path, which is where they
    /// belong: display text reaches the UI without them.
    pub fn warn(&self, context: &str) {
        tracing::warn!(
            category = "storage",
            context,
            operation = ?self.operation,
            path = %self.path.display(),
            error = %self.message,
            "Client storage step failed"
        );
    }
}

#[cfg(test)]
mod tests;
