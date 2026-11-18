//! Local TUI sound settings and endpoint-scoped semantic notification policy.

mod config;
mod error;
mod policy;
#[cfg(test)]
mod tests;

pub use config::{Settings, SoundConfig};
pub use error::{Error, Result};
pub use policy::{Live, MAX_PENDING, Policy};
