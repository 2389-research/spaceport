//! The Settings window's paint kit: the row, the number field, the dropdown and
//! the host colour swatches every page of that window is built from, the bounds
//! its rows report to search, the section list and footer, the scrollbars of
//! its two panes, and the table a dropdown's index or a slider's value is read
//! through.
//!
//! Pure rendering and pure policy. Each builder takes the theme and the values
//! it draws, and answers through the closures it is given; the state a row
//! shows, the scroll position and the saves stay on the window, which is the
//! app's.
//!
//! The words are this crate's: each builder names its own messages and reads
//! them in the language the window shows, through `spaceport-i18n`. An id that
//! arrives as data, such as a catalogued row's title, is read through
//! `spaceport_i18n::message`.

// 2026-11-27 quality gate: 2 functions here are over clippy.toml's 100-line
// ceiling. Split them, then delete this with the crate's entry in
// docs/notes-quality-gate-sweep.md.
#![allow(clippy::too_many_lines)]

pub mod bounds;
pub mod forms;
pub mod inputs;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub mod native;
pub mod navigation;
pub mod scrollbar;
pub mod ui;
