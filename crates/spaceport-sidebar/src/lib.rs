//! The sidebar's core: the geometry its rows are measured with, the density and
//! style a layout name selects and the chrome rows are marked with, the order
//! its workspaces and agents list and step in, the host headers that stay
//! pinned while the list scrolls, which devices it lists and where each one
//! runs, which workspace rows a folded group shows, what a row is and how its
//! two lines are weighted, the inputs every row of one render shares, the
//! trunk, tick and icons a row's leading edge draws, which side that edge is on
//! in a right-to-left language, the rows the daemon's configured tokens resolve
//! to, the props its status marks are drawn with, the status sets drawn rather
//! than loaded from assets, the note and mute marks at a row's trailing edge,
//! the colour wash a host lays over its rows, the local state it draws from —
//! the notes on checkouts and the tags on devices — and the painters: the two
//! lines of a row, its status slot, and the four layouts plus the mode that
//! picks one, all the way down to the element a word is.
//!
//! No window: the app owns every message, the copy every window reads, the file
//! it saves them in, the window's mark layer, and the catalog every word comes
//! from. It hands those three in as [`paint::Painters`] and the shown locale's
//! direction as [`row::RowContext::right_to_left`], and paints with these for
//! everything else.
//! runs, which workspace rows a folded group shows, what a row is and how the
//! two lines of it are weighted, the inputs every row of one render shares, the
//! trunk, tick and icons a row's leading edge draws, the rows the daemon's
//! configured tokens resolve to, the props its status marks are drawn with, the
//! status sets drawn rather than loaded from assets, the note and mute marks at
//! a row's trailing edge, the colour wash a host lays over its rows, the local
//! state it draws from — the notes on checkouts and the tags on devices — and
//! the interaction it answers: the two drags that reorder workspaces and
//! devices, what a resting pointer is claiming, the colour and the grouping a
//! tag brings, and how far the sidebar slides.
//! No window: the app owns every message, the copy every window reads, and the
//! file it saves them in, decides which side a row's leading edge is on, paints
//! every row, and words every status; it reads these for the numbers, the order,
//! the kinds, the marks, the drags, and the users' own words to draw.

mod error;

pub use error::{Error, Result};

pub mod agents;
pub mod animated;
pub mod cell;
pub mod device_drag;
pub mod device_scope;
pub mod device_tags;
pub mod direction;
pub mod hover;
pub mod layout;
pub mod layouts;
pub mod metrics;
pub mod order;
pub mod origin;
pub mod paint;
pub mod reorder;
pub mod row;
pub mod slide;
pub mod sticky;
pub mod tag_groups;
pub mod tokens;
pub mod wash;
pub mod workspaces;
pub mod worktree_notes;
