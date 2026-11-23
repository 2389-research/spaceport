//! The sidebar's pure core: the geometry its rows are measured with, the order
//! its workspaces and agents list and step in, the host headers that stay
//! pinned while the list scrolls, and which workspace rows a folded group
//! shows. No GPUI and no window: the app owns every message, state, and row it
//! draws, and reads these for the numbers and the order to draw them in.

pub mod metrics;
pub mod order;
pub mod sticky;
pub mod workspaces;
