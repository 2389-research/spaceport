//! Client storage errors keep their path out of display text, which reaches
//! the UI. The warning log is where that path belongs, so debugging a failed
//! file read does not need a debugger.
//!
//! The failure itself is `spaceport_endpoint::StorageFailure`: the catalog's
//! two-second retry is what decides which failures it has already named, so the
//! value is compared there and logged through this module.
pub(crate) use spaceport_endpoint::StorageFailure;

use std::error::Error as StdError;

/// Log a failed storage step with its operation and path. Errors without a
/// client storage step in their source chain are not logged.
pub(crate) fn warn_storage_failure(context: &str, error: &(dyn StdError + 'static)) {
    if let Some(failure) = StorageFailure::find(error) {
        failure.warn(context);
    }
}

#[cfg(test)]
mod tests;
