// ABOUTME: Preserves native updater failures and unsupported or busy states.
// ABOUTME: Keeps Sparkle errors typed until the user-facing boundary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(all(target_os = "macos", feature = "macos-updater"))]
    #[error("An update check is already in progress.")]
    Busy,
    #[cfg(all(target_os = "macos", feature = "macos-updater"))]
    #[error(transparent)]
    Sparkle(#[from] sparkle_updater::Error),
    #[cfg(all(target_os = "macos", feature = "macos-updater"))]
    #[error("The compiled preview trust configuration is invalid: {0}")]
    Trust(#[from] serde_json::Error),
    #[error("Updates are available only in an installed, updater-enabled macOS Spaceport preview.")]
    Unsupported,
}

impl Error {
    pub(crate) fn localize(&self) -> String {
        match self {
            Self::Unsupported => crate::i18n::fl!("updates-unsupported"),
            #[cfg(all(target_os = "macos", feature = "macos-updater"))]
            Self::Busy => crate::i18n::fl!("updates-busy"),
            #[cfg(all(target_os = "macos", feature = "macos-updater"))]
            Self::Sparkle(_) | Self::Trust(_) => {
                crate::i18n::fl!("updates-error", error = self.to_string())
            }
        }
    }
}
