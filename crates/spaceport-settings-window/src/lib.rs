//! The Settings window's paint kit: the row, the number field, the dropdown and
//! the host colour swatches every page of that window is built from, and the
//! bounds its rows report to search.
//!
//! Pure rendering. Each builder takes the theme and returns elements; the state
//! a row shows and the saves it makes stay on the window, which is the app's.
//!
//! No words: [`ui::Labels`] is the app's, so a row, a number's unit and a
//! dropdown's label are drawn in the language the window shows without this
//! crate naming one. [`ui::MESSAGE_IDS`] lists the ids it names for the app to
//! word and to check.

pub mod bounds;
pub mod forms;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub mod native;
pub mod ui;

#[cfg(test)]
mod tests;
