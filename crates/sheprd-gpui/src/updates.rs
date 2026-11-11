// ABOUTME: Owns native preview updates for the application's main-thread lifetime.
// ABOUTME: Routes checks through Sparkle's standard consent and installation UI.
mod error;
pub use error::Error;

#[cfg(all(target_os = "macos", feature = "macos-updater"))]
mod native;
#[cfg(any(test, all(target_os = "macos", feature = "macos-updater")))]
mod policy;

pub(crate) fn supported_build() -> bool {
    cfg!(all(target_os = "macos", feature = "macos-updater"))
        && env!("HERDR_BUILD_WORKTREE") == "0"
        && crate::constants::APP_ID == "ai.2389.spaceport"
        && crate::constants::APP_VERSION != "dev"
}

pub(crate) fn install(cx: &mut gpui::App) {
    #[cfg(all(target_os = "macos", feature = "macos-updater"))]
    if let Err(error) = native::install(cx) {
        tracing::warn!(%error, "Native updates unavailable");
    }
    #[cfg(not(all(target_os = "macos", feature = "macos-updater")))]
    let _ = cx;
}

pub(crate) fn available(cx: &gpui::App) -> Result<(), Error> {
    #[cfg(all(target_os = "macos", feature = "macos-updater"))]
    return native::available(cx);
    #[cfg(not(all(target_os = "macos", feature = "macos-updater")))]
    {
        let _ = cx;
        Err(Error::Unsupported)
    }
}

pub(crate) fn check(cx: &gpui::App) -> Result<(), Error> {
    available(cx)?;
    #[cfg(all(target_os = "macos", feature = "macos-updater"))]
    native::check(cx)?;
    Ok(())
}

// Menu construction must not persist a temporary in-flight check as a disabled item.
pub(crate) fn menu_available(cx: &gpui::App) -> bool {
    menu_enabled(available(cx))
}

fn menu_enabled(availability: Result<(), Error>) -> bool {
    match availability {
        Ok(()) => true,
        #[cfg(all(target_os = "macos", feature = "macos-updater"))]
        Err(Error::Busy) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
#[path = "updates/tests/menu.rs"]
mod menu_tests;
