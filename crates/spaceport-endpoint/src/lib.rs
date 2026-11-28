//! The endpoint program's policy and its host, below the app and below GPUI: one
//! host's link machine and the live state it is read from, the saved-host
//! catalog and the device list it describes, the Back and Forward trail through
//! the panes a host has focused, when the machine slept or its network changed,
//! where a dragged device lands in the user's own order, whose focus a
//! background host may take, the connection a deferred action started on, the
//! IDs saved hosts are keyed by, and one host as a row paints it.
//!
//! Nothing here holds a window, a connection or a message, and no line of it is
//! a word: the app keeps every view, every socket and every `fl!` site, and
//! calls these with the values it has. A link change enters as a [`Transition`]
//! and comes back as the [`Action`] the window performs on the socket it keeps.
//! The catalog's loads and writes run on their own threads; what they find is a
//! value the window reconciles its endpoints against.
// The catalog's startup test starts the test binary again; production code
// here starts no process at all.
#![cfg_attr(test, allow(clippy::disallowed_methods))]
// 2026-11-27 quality gate: 1 function here is over clippy.toml's 100-line
// ceiling. Split it, then delete this with the crate's entry in
// docs/notes-quality-gate-sweep.md.
#![allow(clippy::too_many_lines)]

// keep-sorted start
mod catalog;
mod error;
mod follow;
mod host;
mod ids;
mod order;
mod storage;
mod wake;
// keep-sorted end

pub use catalog::{Catalog, CatalogUpdate, Device, devices, entry_changed};
mod localize;

pub use error::{Error, Result};
pub use follow::focus_requested;
#[cfg(any(test, feature = "test-support"))]
pub use host::link::{Failure, Link};
pub use host::link::{MAX_RETRY_DELAY, STABLE_CONNECTION_PERIOD};
pub use host::{
    Action, ConnectionRef, Host, HostView, InputFence, Phase, SelectionFence, Transition,
    Transport, WeakConnectionRef, live_of,
};
pub use ids::{LOCAL, SAVED_PREFIX, WSL_PREFIX, saved_profile_id};
pub use order::{moved, rank};
pub use spaceport_client::pane_history::{History, Step};
pub use storage::StorageFailure;
pub use wake::{NetworkChanges, Trigger, WakeClock, network_changes, watch_network};

#[cfg(test)]
mod tests;
