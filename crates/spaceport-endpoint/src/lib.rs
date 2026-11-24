//! The endpoint program's policy, below the app and below GPUI: the Back and
//! Forward trail through the panes a host has focused, when the machine slept
//! or its network changed, where a dragged device lands in the user's own
//! order, whose focus a background host may take, and the IDs saved hosts are
//! keyed by.
//!
//! Nothing here holds a window, a connection, a message or a word: the app
//! keeps every view, every socket and every `fl!` site, and calls these with
//! the values it has. A function that cannot fail has no `Error`, so the crate
//! has no `error.rs` yet.

// keep-sorted start
mod follow;
mod history;
mod ids;
mod order;
mod wake;
// keep-sorted end

pub use follow::focus_requested;
pub use history::{History, Step};
pub use ids::{LOCAL, SAVED_PREFIX, WSL_PREFIX, saved_profile_id};
pub use order::{moved, rank};
pub use wake::{NetworkChanges, Trigger, WakeClock, network_changes, watch_network};

#[cfg(test)]
mod tests;
