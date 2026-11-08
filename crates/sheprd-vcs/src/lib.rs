//! Git on the checkouts Sheprd shows: working-tree status and the explicit
//! writes (commit, push, pull request creation). Every operation runs a child
//! process, so callers queue work on these workers and never run it on the UI
//! thread; no GPUI.
//!
//! Source indexing is outside this crate's public API:
//! ```compile_fail
//! use sheprd_vcs::code_index;
//! ```

#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod error;
pub mod git;

pub use error::{Error, Result};
