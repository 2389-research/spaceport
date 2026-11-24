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
//! No words: [`ui::Labels`] is the app's, so a row, a number's unit and a
//! dropdown's label are drawn in the language the window shows without this
//! crate naming one. [`ui::MESSAGE_IDS`] lists the ids it names for the app to
//! word and to check.

pub mod bounds;
pub mod forms;
pub mod inputs;
#[cfg(all(feature = "integration-test", target_os = "macos"))]
pub mod native;
pub mod navigation;
pub mod scrollbar;
pub mod ui;

#[cfg(test)]
mod tests;
