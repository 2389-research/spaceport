// ABOUTME: Defines build identity and fixed window presentation values.
// ABOUTME: Keeps development and signed preview metadata tied to their build inputs.
//! Values fixed at build time or by the reference design: the development app
//! identity the OS sees, the tab strip's geometry, and the release version.

// Every window carries the product name; the focused space follows it.
pub(crate) const WINDOW_TITLE: &str = sheprd_client::APP_NAME;

/// The external runtime that owns daemon announcements and release notes.
pub(crate) const HERDR_NAME: &str = "Herdr"; // i18n: data

// Development windows and notifications must not share Sheprd's OS identity.
// Release builds require verified independent production identities.
pub(crate) const APP_ID: &str = env!("SPACEPORT_APP_ID");
// Keep the inherited release route closed until Spaceport's identity is configured.
const _: () = assert!(
    option_env!("HERDR_RELEASE_VERSION").is_none(),
    "Spaceport release identity is not configured"
);

/// What kind of build this is, decided once by the build script's
/// `preview_identity::select`. Code that behaves differently in a signed
/// preview asks this, never the app ID or version strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Channel {
    /// An ordinary local or worktree build.
    Development,
    /// The signed macOS preview route.
    MacosPreview,
}

impl Channel {
    /// The channel the build script names, or `None` for any other text.
    pub(crate) const fn parse(name: &str) -> Option<Self> {
        match name.as_bytes() {
            b"development" => Some(Self::Development),
            b"macos-preview" => Some(Self::MacosPreview),
            _ => None,
        }
    }
}

// An unknown channel stops the build: there is no runtime fallback.
#[cfg_attr(
    not(all(target_os = "macos", feature = "macos-updater")),
    allow(dead_code, reason = "only the native updater reads the channel so far")
)]
pub(crate) const CHANNEL: Channel = match Channel::parse(env!("SPACEPORT_BUILD_CHANNEL")) {
    Some(channel) => channel,
    None => panic!("SPACEPORT_BUILD_CHANNEL is not a known build channel"),
};

/// Preview builds carry an explicit version; ordinary local builds say dev.
pub(crate) const APP_VERSION: &str = env!("SPACEPORT_APP_VERSION");

// Wayland app ID and X11 WM_CLASS. Linux desktop metadata uses the development
// identity; production packaging remains blocked.
pub(crate) const LINUX_APP_ID: &str = APP_ID;

// Even tab cells, as on herdr.dev, so short labels do not collapse to a sliver.
pub(crate) const TAB_WIDTH: f32 = 64.;
// The reference strip is a shallow band: chrome, not a toolbar.
pub(crate) const TAB_HEIGHT: f32 = 24.;

// The git commit hash embedded by build.rs for source/dev builds.
pub(crate) const BUILD_COMMIT: Option<&str> = option_env!("HERDR_BUILD_COMMIT");

#[cfg(test)]
mod tests;
