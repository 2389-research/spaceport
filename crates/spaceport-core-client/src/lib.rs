#![doc = include_str!("../README.md")]
// Tests start real processes; production code builds them with `new_command`.
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod backend;
mod connection;
mod endpoint;
mod error;
pub mod event;
pub mod frame;
pub mod hello;
mod method;
mod start;
mod stream;
pub mod wire;

#[cfg(test)]
// 2026-11-27 quality gate: 6 functions in this module are over clippy.toml's
// 100-line ceiling and 1 over its cognitive-complexity ceiling. Split them,
// then delete this with the crate's entry in docs/notes-quality-gate-sweep.md.
#[allow(
    clippy::cognitive_complexity,
    clippy::expect_used,
    clippy::too_many_lines,
    clippy::unwrap_used
)]
mod tests;

#[cfg(all(test, unix))]
#[path = "../../test-support/executable.rs"]
mod test_executable;

pub use backend::{CoreBackend, CoreBatch, CoreChanges, CoreTerminal, DocumentInvalidation};
pub use connection::{Core, MAX_PUT_BYTES};
pub use endpoint::Endpoint;
pub use error::{Error, RefusalHint, Result, Violation};
pub use method::Method;
pub use start::{PROGRAM, start};
pub use stream::{Attachment, Chunk, Closed, End, MAX_BUFFERED, Subscription};
