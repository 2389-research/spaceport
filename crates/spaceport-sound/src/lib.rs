//! Local TUI sound settings, the endpoint-scoped semantic notification policy,
//! and the worker that plays what it asks for.
//!
//! Policy and playback adapted from Herdr; see SOUND-NOTICE.md.
// Tests start real processes; production code builds them with `new_command`.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod config;
mod error;
mod localize;
mod playback;
mod policy;
mod service;
#[cfg(test)]
mod tests;

pub use error::{Error, Result};
pub use playback::{MAX_VOLUME, muted};
pub use policy::{Live, MAX_PENDING, Policy};
pub use service::Service;
