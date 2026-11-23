//! The sidebar's pure core: the geometry its rows are measured with, the order
//! its workspaces and agents list and step in, the host headers that stay
//! pinned while the list scrolls, which workspace rows a folded group shows,
//! and the local state it draws from — the notes on checkouts and the tags on
//! devices. No GPUI and no window: the app owns every message, the copy every
//! window reads, and the file it saves them in, and reads these for the
//! numbers, the order, and the users' own words to draw.

mod error;

pub use error::{Error, Result};

pub mod device_tags;
pub mod metrics;
pub mod order;
pub mod sticky;
pub mod workspaces;
pub mod worktree_notes;
