// ABOUTME: Shares localization assertions across typed error families.
// ABOUTME: Checks English diagnostics and complete pseudo-locale messages.
//! Shared checks for every error mapping, and a topic file per app-side
//! mapping. A crate's own errors are checked in the crate that words them.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::Localize;
use spaceport_i18n::{Locale, Pseudo};

mod backdrop;
mod caffeine;
mod clipboard;
mod daemon;
mod endpoint;
mod file_drop;
#[cfg(windows)]
mod launch_at_login;
mod listening_ports;
mod notification_mutes;
mod overview;
mod pane_move;
mod port_forward;
mod processes;
mod root;
mod wsl_drop;

pub(crate) use spaceport_i18n::assertions::{
    assert_localized, assert_no_wire_words, assert_verbatim,
};
