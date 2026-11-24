// ABOUTME: Holds a connection's notices at the path the window reads them at.
// ABOUTME: The typed notices and their lifetimes live in spaceport-live.
//! The window's typed notices — one current line with a bounded lifetime, and
//! the held one input replay restores — are the live-state crate's, beside the
//! state whose status bar shows them.
pub(crate) use spaceport_live::notice::{Cause, HeldNotice, Notices};
