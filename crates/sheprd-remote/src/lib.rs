//! The machines Sheprd reaches beyond this computer's own daemon: running
//! scripts on any host and Teleport, which moves a session's work. No
//! GPUI: the app runs these on its own background workers and shows their
//! progress and dialogs.
// Tests start fixture processes directly; production children use
// sheprd_client::new_command.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod error;
pub mod teleport;

pub use error::{Error, Result};
