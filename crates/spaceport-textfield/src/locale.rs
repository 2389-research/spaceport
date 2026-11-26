//! The app's notice that the shown locale changed, so a field built before
//! the switch formats its placeholder again. The field words nothing itself:
//! its caller passes the closure that formats the placeholder, and this tells
//! that closure when to run.

use gpui::{App, Global};

/// Tells the fields already built that the locale changed, as the app
/// publishes a direction to `spaceport-widgets`. The count only has to differ
/// from the last one; a field reads nothing from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocaleChanged(u64);

impl Global for LocaleChanged {}

impl LocaleChanged {
    /// Publishes a locale change to the fields shown, and to those built
    /// after it.
    pub fn set(cx: &mut App) {
        let count = cx.try_global::<Self>().map_or(1, |last| last.0 + 1);
        cx.set_global(Self(count));
    }
}
