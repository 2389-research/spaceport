//! ABOUTME: Shared host scripts and agent metadata for retained desktop routes.
//! ABOUTME: Keeps Herdr target selection independent of GPUI and transport ownership.

pub mod agent;
mod error;
pub mod host_script;
pub mod snapshot;

pub use error::{Error, Result};
