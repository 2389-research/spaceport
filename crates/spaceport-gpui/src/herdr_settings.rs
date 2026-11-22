// ABOUTME: Names Herdr's shared settings, which live in the settings crate.
// ABOUTME: Keeps the path the app has always read them by.
//! Herdr's settings schema, kept in Spaceport's own files: `config.toml` holds
//! this app's managed defaults, `config.local.toml` its overrides, and the
//! theme, sound, toast, copy-on-select, tab-bar and pane-history choices the
//! Settings window shares with Herdr are read and written in
//! `spaceport/view.toml`. The runtime's own `herdr/config.toml` is only read,
//! for the few keys the app honors.
//!
//! The model, its guarded writes and its remote read-modify-write live in
//! `spaceport-settings`; this is the path the app reads them by.
pub(crate) use spaceport_settings::herdr_settings::*;
