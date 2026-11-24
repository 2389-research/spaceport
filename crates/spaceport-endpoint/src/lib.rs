//! The endpoint program's policy, below the app and below GPUI: the saved-host
//! catalog and the device list it describes, the Back and Forward trail through
//! the panes a host has focused, when the machine slept or its network changed,
//! where a dragged device lands in the user's own order, whose focus a
//! background host may take, and the IDs saved hosts are keyed by.
//!
//! Nothing here holds a window, a connection or a message, and no line of it is
//! a word: the app keeps every view, every socket and every `fl!` site, and
//! calls these with the values it has. The catalog's loads and writes run on
//! their own threads; what they find is a value the window reconciles its
//! endpoints against.
// The catalog's startup test starts the test binary again; production code
// here starts no process at all.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

// keep-sorted start
mod catalog;
mod error;
mod follow;
mod history;
mod ids;
mod order;
mod storage;
mod wake;
// keep-sorted end

pub use catalog::{Catalog, CatalogUpdate, Device, devices, entry_changed};
pub use error::{Error, Result};
pub use follow::focus_requested;
pub use history::{History, Step};
pub use ids::{LOCAL, SAVED_PREFIX, WSL_PREFIX, saved_profile_id};
pub use order::{moved, rank};
pub use storage::StorageFailure;
pub use wake::{NetworkChanges, Trigger, WakeClock, network_changes, watch_network};

#[cfg(test)]
mod tests;
