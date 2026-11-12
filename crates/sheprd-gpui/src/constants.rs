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

/// Preview builds carry an explicit version; ordinary local builds say dev.
pub(crate) const APP_VERSION: &str = env!("SPACEPORT_APP_VERSION");

// Wayland app ID and X11 WM_CLASS. The inherited release desktop entry does
// not match this development-only ID; production packaging remains blocked.
pub(crate) const LINUX_APP_ID: &str = APP_ID;

// Even tab cells, as on herdr.dev, so short labels do not collapse to a sliver.
pub(crate) const TAB_WIDTH: f32 = 64.;
// The reference strip is a shallow band: chrome, not a toolbar.
pub(crate) const TAB_HEIGHT: f32 = 24.;

// Legacy production-build policy for automatic skill management and issue reporting.
// Signed macOS previews keep this false.
pub(crate) const RELEASE_BUILD: bool = option_env!("HERDR_RELEASE_VERSION").is_some();

// The git commit hash embedded by build.rs for source/dev builds.
pub(crate) const BUILD_COMMIT: Option<&str> = option_env!("HERDR_BUILD_COMMIT");

#[cfg(test)]
mod tests;
