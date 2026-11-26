//! The platform input handler a view-backed text field hands GPUI, chosen for
//! the desktop it runs on.

#[cfg(any(target_os = "linux", windows))]
mod weak;

// The platform input handler for every view-backed text field. It is not named
// ElementInputHandler, so a `gpui::*` glob cannot silently replace it with
// GPUI's strong handler on Linux or Windows.
#[cfg(not(any(target_os = "linux", windows)))]
pub use gpui::ElementInputHandler as ViewInputHandler;
#[cfg(any(target_os = "linux", windows))]
pub use weak::WeakInputHandler as ViewInputHandler;
