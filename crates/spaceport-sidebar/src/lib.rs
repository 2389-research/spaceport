//! The sidebar's core: the geometry its rows are measured with, the density and
//! style a layout name selects and the chrome rows are marked with, the order
//! its workspaces and agents list and step in, the host headers that stay
//! pinned while the list scrolls, which workspace rows a folded group shows,
//! what a row is and how the two lines of it are weighted, the props its status
//! marks are drawn with, the status sets drawn rather than loaded from assets,
//! the note and mute marks at a row's trailing edge, the colour wash a host lays
//! over its rows, and the local state it draws from — the notes on checkouts
//! and the tags on devices. No window: the app owns every message, the copy
//! every window reads, and the file it saves them in, paints every row, and
//! words every status; it reads these for the numbers, the order, the kinds,
//! the marks, and the users' own words to draw.

mod error;

pub use error::{Error, Result};

pub mod agents;
pub mod animated;
pub mod device_tags;
pub mod layout;
pub mod metrics;
pub mod order;
pub mod row;
pub mod sticky;
pub mod tokens;
pub mod wash;
pub mod workspaces;
pub mod worktree_notes;
