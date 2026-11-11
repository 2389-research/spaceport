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
