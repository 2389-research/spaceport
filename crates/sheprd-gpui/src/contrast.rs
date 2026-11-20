//! How far the app's own colors stand off the chrome they are drawn on: the
//! contrast levels a theme chooses and the WCAG rules that reach them.
//!
//! The rules live with the settings they read; this is the path the app's
//! painters have always named them by.
pub(crate) use sheprd_settings::contrast::*;

#[cfg(all(windows, feature = "integration-test"))]
mod native;
#[cfg(all(windows, feature = "integration-test"))]
pub(crate) use native::verify_secondary_text;
