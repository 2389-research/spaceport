// ABOUTME: Validates the running preview bundle before starting native updates.
// ABOUTME: Keeps build identity and signed bundle trust settings consistent.
use crate::constants::Channel;

pub(super) struct Bundle<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub path: &'a str,
    pub feed: &'a str,
    pub key: &'a str,
    pub signed_feed: bool,
    pub signed_feed_failure_expiration: Option<f64>,
    pub verify_before_extraction: bool,
    pub automatic_install: bool,
    pub automatic_checks_configured: bool,
}

/// The identity a build was compiled with.
pub(super) struct Build<'a> {
    pub channel: Channel,
    pub id: &'a str,
    pub version: &'a str,
    /// The build script's worktree marker: `"0"` outside a linked worktree.
    pub worktree: &'a str,
}

impl Build<'static> {
    /// This executable's own identity, from the build script.
    pub(super) const COMPILED: Self = Self {
        channel: crate::constants::CHANNEL,
        id: crate::constants::APP_ID,
        version: crate::constants::APP_VERSION,
        worktree: env!("HERDR_BUILD_WORKTREE"),
    };
}

impl Build<'_> {
    /// Whether this kind of build may update at all: a signed macOS preview
    /// built outside a worktree. Development builds never update, whatever
    /// their ID or version text says.
    pub(super) fn can_update(&self) -> bool {
        self.channel == Channel::MacosPreview && self.worktree == "0"
    }
}

/// Whether the running bundle is the installed copy of `build`, carrying the
/// compiled trust settings. This is the only answer to "may this app update".
pub(super) fn eligible(bundle: &Bundle<'_>, build: &Build<'_>, feed: &str, key: &str) -> bool {
    build.can_update()
        && !build.version.is_empty()
        && bundle.id == build.id
        && bundle.version == build.version
        && std::path::Path::new(bundle.path)
            .extension()
            .is_some_and(|ext| ext == "app")
        && bundle.feed == feed
        && bundle.key == key
        && bundle.signed_feed
        && bundle.signed_feed_failure_expiration == Some(0.0)
        && bundle.verify_before_extraction
        && !bundle.automatic_install
        && !bundle.automatic_checks_configured
}

#[cfg(test)]
#[path = "tests/policy.rs"]
mod tests;
