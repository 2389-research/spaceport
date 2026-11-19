// ABOUTME: Names Herdr's shared settings, which live in the settings crate.
// ABOUTME: Keeps the path the app has always read them by.
//! Herdr's shared release-namespace settings: `spaceport`'s `config.toml` is
//! the managed defaults and `herdr/config.toml` is edited beside it.
//!
//! The model, its guarded writes and its remote read-modify-write live in
//! `sheprd-settings`; this is the path the app reads them by.
pub(crate) use sheprd_settings::herdr_settings::*;
