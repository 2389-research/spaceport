//! What Settings reads of the window it was opened from.
//!
//! Settings is opened from a main window and shows facts that window owns: the
//! config, keymap and theme it started editing, Herdr's shared settings it
//! loaded, the target a new window would connect to, whether its own
//! integrations listing is in flight, whether its theme picker is open, and
//! the hosts it is connected to. Settings reads them here instead of through
//! the window entity, so no page or control holds that window.
//!
//! The window observes its source (`SettingsWindow::source_changed`), and that
//! callback follows the facts that keep changing. The appearance it starts from
//! does not: what Settings edits from it is its own draft, and re-reading it
//! would undo the draft under the user's hands.

use crate::SpaceportWindow;
use crate::config::{Config, Theme};
use spaceport_client::ConnectTarget;
use spaceport_keymap::Keymap;

/// One connected endpoint, as the pages that list hosts show it.
pub(super) struct SourceHost {
    /// The name the sidebar's host header shows, and the key its colour is
    /// saved under.
    pub(super) label: String,
    /// What that endpoint connects to, which names an SSH host's config file.
    pub(super) target: ConnectTarget,
}

/// The window Settings was opened from, as one value.
pub(super) struct SourceWindow {
    /// The config it was showing when Settings opened, which the editable
    /// draft starts from.
    pub(super) config: Config,
    /// The keymap that config resolved into, so a published appearance carries
    /// a settings and keymap pair that always agree.
    pub(super) keymap: Keymap,
    /// The theme it was showing when Settings opened.
    pub(super) theme: Theme,
    /// Herdr's shared settings as it had loaded them.
    pub(super) shared: Option<crate::herdr_settings::Settings>,
    /// The target its first endpoint connects to: where "New window" goes, and
    /// all Settings keeps of it once the source closes.
    pub(super) target: Option<ConnectTarget>,
    /// Whether its own integrations listing is in flight, which fences
    /// retargeting away from it mid-listing.
    pub(super) integrations_busy: bool,
    /// Whether its theme picker is open or saving: a hover preview is live, so
    /// a published appearance must not reach the log window yet.
    pub(super) theme_page_open: bool,
    /// The endpoints it is connected to, in window order.
    pub(super) hosts: Vec<SourceHost>,
}

impl SourceWindow {
    /// Everything Settings reads of `window`, for the moment it opens.
    pub(super) fn of(window: &SpaceportWindow) -> Self {
        Self {
            config: window.config.clone(),
            keymap: window.keymap.clone(),
            theme: window.theme.clone(),
            shared: window.settings.shared.clone(),
            target: target(window),
            integrations_busy: window.integrations.busy,
            theme_page_open: theme_page_open(window),
            hosts: hosts(window),
        }
    }

    /// The app's published appearance, for a Settings window whose source is
    /// gone before it opened: there is no window left to read.
    pub(super) fn from_appearance(appearance: &crate::app::InitialAppearance) -> Self {
        Self {
            config: appearance.shown.config.clone(),
            keymap: appearance.keymap.clone(),
            theme: appearance.shown.theme.clone(),
            shared: None,
            target: None,
            integrations_busy: false,
            theme_page_open: false,
            hosts: Vec::new(),
        }
    }

    /// The facts that follow `window` while Settings is open. The appearance
    /// above stays as it was when Settings opened; these are what it shows now.
    pub(super) fn follow(&mut self, window: &SpaceportWindow) {
        self.target = target(window);
        self.integrations_busy = window.integrations.busy;
        self.theme_page_open = theme_page_open(window);
        self.hosts = hosts(window);
    }

    /// The window has closed. Nothing is listed as connected any more, and the
    /// appearance and target it left behind stay readable.
    pub(super) fn release(&mut self) {
        self.integrations_busy = false;
        self.theme_page_open = false;
        self.hosts.clear();
    }
}

fn target(window: &SpaceportWindow) -> Option<ConnectTarget> {
    window
        .endpoints
        .first()
        .map(|endpoint| endpoint.connection.target.clone())
}

fn theme_page_open(window: &SpaceportWindow) -> bool {
    window.menu.page == Some(crate::menu::Page::Themes) || window.theme_save_in_flight()
}

/// The endpoints that are connected, which are the hosts a page may colour or
/// write a config file on.
fn hosts(window: &SpaceportWindow) -> Vec<SourceHost> {
    window
        .endpoints
        .iter()
        .filter(|endpoint| endpoint.live().status.is_connected())
        .map(|endpoint| SourceHost {
            label: endpoint.label.clone(),
            target: endpoint.connection.target.clone(),
        })
        .collect()
}
