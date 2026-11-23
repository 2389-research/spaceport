//! The sidebar's core: the geometry its rows are measured with, the density and
//! style a layout name selects and the chrome rows are marked with, the order
//! its workspaces and agents list and step in, the host headers that stay
//! pinned while the list scrolls, which devices it lists and where each one
//! runs, which workspace rows a folded group shows, what a row is and how the
//! two lines of it are weighted, the inputs every row of one render shares, the
//! trunk, tick and icons a row's leading edge draws, which side that edge is on
//! in a right-to-left language, how it reaches the text, moving marks and words
//! only the app can paint, the rows the daemon's
//! configured tokens resolve to, the props its status marks are drawn with, the
//! status sets drawn rather than loaded from assets, the note and mute marks at
//! a row's trailing edge, the colour wash a host lays over its rows, and the
//! local state it draws from — the notes on checkouts and the tags on devices.
//! No window: the app owns every message, the copy every window reads, and the
//! file it saves them in, decides which side a row's leading edge is on, paints
//! every row, and words every status; it reads these for the numbers, the order,
//! the kinds, the marks, and the users' own words to draw.

mod error;

pub use error::{Error, Result};

pub mod agents;
pub mod animated;
pub mod cell;
pub mod device_scope;
pub mod device_tags;
pub mod direction;
pub mod layout;
pub mod metrics;
pub mod order;
pub mod origin;
pub mod paint;
pub mod row;
pub mod sticky;
pub mod tokens;
pub mod wash;
pub mod workspaces;
pub mod worktree_notes;
