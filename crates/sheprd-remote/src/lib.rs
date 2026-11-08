//! The machines Sheprd reaches beyond this computer's own daemon: the cloud
//! providers that create them (each behind its feature), running scripts on
//! any host, and Teleport, which moves a session's work to another host. No
//! GPUI: the app runs these on its own background workers and shows their
//! progress and dialogs.
// Tests start fixture processes directly; production children use
// sheprd_client::new_command.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "cloud")]
pub mod cloud;
pub mod config;
#[cfg(feature = "container")]
pub mod container;
#[cfg(feature = "azure")]
mod credentials;
mod error;
pub mod teleport;

pub use error::{Error, Result};
