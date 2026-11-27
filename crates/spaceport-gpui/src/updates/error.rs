// ABOUTME: Preserves native updater failures and unsupported or busy states.
// ABOUTME: Keeps Sparkle and trust errors typed until the user-facing boundary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An update check is already in progress.")]
    Busy,
    #[error("A compiled update feed URL is not an https address: {0}")]
    InsecureFeed(String),
    #[error("The compiled update trust configuration names no public key.")]
    MissingPublicKey,
    #[cfg(all(target_os = "macos", feature = "macos-updater"))]
    #[error(transparent)]
    Sparkle(#[from] sparkle_updater::Error),
    #[error("The compiled update trust configuration is invalid: {0}")]
    Trust(#[from] serde_json::Error),
    #[error("Updates are available only in an installed, updater-enabled macOS Spaceport release.")]
    Unsupported,
}

impl Error {
    pub(crate) fn localize(&self) -> String {
        match self {
            Self::Unsupported => crate::i18n::fl!("updates-unsupported"),
            Self::Busy => crate::i18n::fl!("updates-busy"),
            Self::InsecureFeed(_) | Self::MissingPublicKey | Self::Trust(_) => {
                crate::i18n::fl!("updates-error", error = self.to_string())
            }
            #[cfg(all(target_os = "macos", feature = "macos-updater"))]
            Self::Sparkle(_) => crate::i18n::fl!("updates-error", error = self.to_string()),
        }
    }
}
