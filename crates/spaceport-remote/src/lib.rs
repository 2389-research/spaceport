//! ABOUTME: Shared host scripts, device catalog edits and agent metadata for retained desktop routes.
//! ABOUTME: Keeps Herdr target selection independent of GPUI and transport ownership.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

pub mod agent;
mod error;
pub mod host_script;
mod localize;
pub mod setup;
pub mod snapshot;

#[cfg(all(test, unix))]
#[path = "../../test-support/executable.rs"]
mod test_executable;

pub use error::{Error, Result};
