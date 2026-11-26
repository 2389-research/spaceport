//! The About box: build identity and attribution, laid out as a demoscene NFO
//! file over a demo stage. It reads embedded constants, never the daemon or
//! the network; links open in the browser.
//!
//! The words are the panel's own — the version, the publisher, the links and
//! the credits, each row one message [`nfo`] lays out — and the scene behind
//! them is decoration: [`demos`] picks which one this launch shows and
//! [`scene`] paints it. `SPACEPORT_ABOUT_CLASSIC` asks for the same document
//! without the scene, for QA.
// i18n: migrated
use crate::{SpaceportWindow, menu::Page};
use gpui::{prelude::*, *};

mod demos;
mod effects;
mod nfo;
mod scene;

pub(crate) use demos::Demo;

pub(super) const WEBSITE: &str = "https://herdr.dev/";
pub(super) const REPOSITORY: &str = "https://github.com/2389-research/spaceport";
const UPSTREAM_REPOSITORY: &str = "https://github.com/sheprd-labs/sheprd";
pub(super) const SHEPRD_NAME: &str = "Sheprd"; // i18n: data
pub(super) const COPYRIGHT_YEAR: &str = "© 2026";
// i18n: data
pub(super) const AUTHOR: &str = "Fabien Penso";
// i18n: data
pub(super) const PUBLISHER: &str = "2389 Research, Inc.";
// i18n: data
pub(super) const LICENSE: &str = "Apache-2.0";
const LICENSE_URL: &str = "https://www.apache.org/licenses/LICENSE-2.0";
/// The third-party notice the license row points at.
pub(super) const NOTICE: &str = "https://github.com/2389-research/spaceport/blob/main/NOTICE";

impl SpaceportWindow {
    pub(super) fn open_about(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open_menu(window, cx) {
            return;
        }
        // One demo a launch: picked here, and kept for as long as the box is
        // open, so a redraw never changes the scene under the reader.
        self.about_demo = demos::choose();
        self.menu.page = Some(Page::About);
    }

    pub(super) fn render_about(&self, cx: &mut Context<Self>) -> Div {
        let demo = self.about_demo;
        let credits = self.render_about_nfo(demo, cx);
        scene::panel(
            credits,
            &self.theme,
            self.config.ui.size,
            scene::classic(),
            demo,
        )
    }
}

#[cfg(test)]
mod tests;
