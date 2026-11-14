// ABOUTME: Declares every static Settings row once, with the state that makes it available.
// ABOUTME: Pages build their rows from these entries and search matches the same text.
//! Every static Settings row, declared once. A page builds a row from its
//! entry with `ui::row`, so the id, title and description a page shows are
//! the ones search matches, and the entry's availability decides both
//! whether the page renders the row and whether search offers it. Rows built
//! from data, such as one per SSH host, have no entry and use `ui::data_row`.
//!
//! Search lists its results in this order, so the order is shown to users:
//! add an entry where its row sits among its page's rows, and do not sort.
// i18n: migrated
use super::{Section, controls::face_message};
use crate::{
    config::{Feature, FontFace},
    i18n,
};

pub(super) struct Entry {
    pub section: Section,
    pub id: &'static str,
    /// The message naming the row.
    pub title: &'static str,
    /// The message describing it, or empty when it has none.
    pub description: &'static str,
    /// When the page renders this row; search offers it only then.
    pub availability: Availability,
}

/// The state a row needs to be rendered at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Availability {
    Always,
    /// Only while the ACP chats feature is on.
    AcpChats,
    /// Only when the running app opted into Herdr Core at startup.
    Core,
    /// Only while the drafted theme follows the system as a light/dark pair.
    FollowingSystemTheme,
    /// Only while ACP chats are on and chats are not full width.
    LimitedChatWidth,
    /// Only on Linux, where the app can draw its own window controls.
    Linux,
    /// Only on Windows, which has the only platform code for it.
    Windows,
}

impl Entry {
    pub(super) fn title(&self) -> String {
        i18n::message(self.title, &[])
    }

    /// The description in the current locale, empty when the row has none.
    pub(super) fn description(&self) -> String {
        if self.description.is_empty() {
            return String::new();
        }
        i18n::message(self.description, &[])
    }

    const fn only(self, availability: Availability) -> Self {
        Self {
            availability,
            ..self
        }
    }
}

const fn entry(
    section: Section,
    id: &'static str,
    title: &'static str,
    description: &'static str,
) -> Entry {
    Entry {
        section,
        id,
        title,
        description,
        availability: Availability::Always,
    }
}

/// A `[features]` flag's row on the General page, which the page builds one
/// of per flag.
pub(super) const fn feature(feature: Feature) -> Entry {
    entry(
        Section::General,
        feature.setting_id(),
        feature.message(),
        feature.description_message(),
    )
}

/// Declares each entry as a named constant, for the page that builds its
/// row, and lists them all in [`CATALOG`] in the order written.
macro_rules! catalog {
    ($($name:ident = $entry:expr;)+) => {
        $(pub(super) const $name: Entry = $entry;)+

        /// Every entry, in the order search lists its results.
        pub(super) const CATALOG: &[Entry] = &[$($name),+];
    };
}

catalog! {
    THEME_MODE = entry(Section::Theme, "theme-mode", "settings-theme-mode", "settings-theme-mode-hint");
    THEME_SIDE = entry(Section::Theme, "theme-side", "settings-theme-side", "settings-theme-side-description").only(Availability::FollowingSystemTheme);
    THEME_FOLLOW = entry(Section::Theme, "theme-follow", "settings-theme-follow", "settings-theme-follow-description");
    THEME_CONTRAST = entry(Section::Theme, "theme-contrast", "settings-theme-contrast", "settings-theme-contrast-description");
    THEME_SOURCE = entry(Section::Theme, "theme-source", "settings-theme-source", "settings-theme-source-description");
    THEME_BACKDROP = entry(Section::Theme, "theme-backdrop", "settings-theme-backdrop", "settings-theme-backdrop-description");
    THEME_PADDING_LEFT = entry(Section::Theme, "theme-padding-left", "settings-theme-padding-left", "settings-theme-padding-left-hint");
    THEME_PADDING_RIGHT = entry(Section::Theme, "theme-padding-right", "settings-theme-padding-right", "settings-theme-padding-right-description");
    THEME_PADDING_TOP = entry(Section::Theme, "theme-padding-top", "settings-theme-padding-top", "settings-theme-padding-top-description");
    THEME_PADDING_BOTTOM = entry(Section::Theme, "theme-padding-bottom", "settings-theme-padding-bottom", "settings-theme-padding-bottom-description");
    SIDEBAR_LAYOUT = entry(Section::Sidebar, "sidebar-layout", "settings-sidebar-layout", "settings-sidebar-layout-description");
    SIDEBAR_ACTIVITY_BAR = entry(Section::Sidebar, "sidebar-activity-bar", "settings-sidebar-activity-bar-title", "settings-sidebar-activity-bar-hint");
    SIDEBAR_SELECT = entry(Section::Sidebar, "sidebar-select", "settings-sidebar-select", "settings-sidebar-select-description");
    SIDEBAR_DEVICE_FOOTER = entry(Section::Sidebar, "sidebar-device-footer", "settings-sidebar-device-footer", "settings-sidebar-device-footer-description");
    SIDEBAR_AUTOMATIONS = entry(Section::Sidebar, "sidebar-automations", "settings-sidebar-automations-title", "settings-sidebar-automations-hint").only(Availability::Core);
    SIDEBAR_SHOW_AGENTS = entry(Section::Sidebar, "sidebar-show-agents", "settings-sidebar-show-agents", "settings-sidebar-show-agents-description");
    SIDEBAR_AGENTS_PLACEMENT = entry(Section::Sidebar, "sidebar-agents-placement", "settings-sidebar-agents-placement", "settings-sidebar-agents-placement-description");
    SIDEBAR_SHOW_AGENTS_HEADER = entry(Section::Sidebar, "sidebar-show-agents-header", "settings-sidebar-show-agents-header", "settings-sidebar-agents-header-hint");
    SIDEBAR_SHOW_SEARCH = entry(Section::Sidebar, "sidebar-show-search", "settings-sidebar-show-search", "settings-sidebar-show-search-description");
    SIDEBAR_REPOSITORY_ICONS = entry(Section::Sidebar, "sidebar-repository-icons", "settings-sidebar-repository-icons", "settings-sidebar-repository-icons-description");
    SIDEBAR_DEVICE_ICONS = entry(Section::Sidebar, "sidebar-device-icons", "settings-sidebar-device-icons", "settings-sidebar-device-icons-description");
    SIDEBAR_INDENT = entry(Section::Sidebar, "sidebar-indent", "settings-sidebar-indent", "settings-sidebar-indent-description");
    SIDEBAR_ROW_PADDING = entry(Section::Sidebar, "sidebar-row-padding", "settings-sidebar-row-padding", "settings-sidebar-row-padding-description");
    SIDEBAR_ROW_GAP = entry(Section::Sidebar, "sidebar-row-gap", "settings-sidebar-row-gap", "settings-sidebar-row-gap-description");
    SIDEBAR_HOST_GAP = entry(Section::Sidebar, "sidebar-host-gap", "settings-sidebar-host-gap", "settings-sidebar-host-gap-description");
    SIDEBAR_HOST_COLOURS = entry(Section::Sidebar, "sidebar-host-colours", "settings-sidebar-host-colours", "settings-sidebar-host-colours-description");
    INDICATORS_EDGE_RAIL = entry(Section::Indicators, "indicators-edge-rail", "settings-indicators-edge-rail", "settings-indicators-edge-rail-description");
    INDICATORS_SETS = entry(Section::Indicators, "indicators-sets", "settings-indicators-sets", "settings-indicators-sets-description");
    FONTS_TERMINAL = entry(Section::Fonts, "fonts-terminal", face_message(FontFace::Terminal), "");
    FONTS_SIDEBAR = entry(Section::Fonts, "fonts-sidebar", face_message(FontFace::Sidebar), "");
    FONTS_TABS = entry(Section::Fonts, "fonts-tabs", face_message(FontFace::Tabs), "");
    FONTS_UI = entry(Section::Fonts, "fonts-ui", face_message(FontFace::Ui), "");
    FONTS_ALL_ROW = entry(Section::Fonts, "fonts-all-row", "settings-fonts-all-row", "");
    STATUS_USAGE = entry(Section::StatusBar, "status-usage", "settings-status-usage", "settings-status-bar-usage-hint");
    STATUS_NETWORK_QUALITY = entry(Section::StatusBar, "status-network-quality", "settings-status-network-quality", "settings-status-network-quality-description");
    STATUS_KEEP_AWAKE = entry(Section::StatusBar, "status-keep-awake", "settings-status-keep-awake", "settings-status-keep-awake-description");
    STATUS_THEME = entry(Section::StatusBar, "status-theme", "settings-status-theme", "");
    STATUS_SHORTCUTS = entry(Section::StatusBar, "status-shortcuts", "settings-status-shortcuts", "");
    STATUS_REPORT_ISSUE = entry(Section::StatusBar, "status-report-issue", "settings-status-report-issue", "");
    GENERAL_LANGUAGE = entry(Section::General, "general-language", "settings-general-language", "settings-general-language-description");
    GENERAL_PORTS = entry(Section::General, "general-ports", "settings-general-ports", "settings-general-ports-description");
    GENERAL_SMOOTH_SCROLL = entry(Section::General, "general-smooth-scroll", "settings-general-smooth-scroll", "settings-general-smooth-scroll-description");
    GENERAL_WINDOW_CONTROLS = entry(Section::General, "general-window-controls", "settings-general-window-controls", "settings-general-window-controls-description").only(Availability::Linux);
    GENERAL_HERDR_CORE = entry(Section::General, "general-herdr-core", "settings-general-herdr-core", "settings-general-herdr-core-description");
    // One per `[features]` flag; a test checks none is missing.
    // keep-sorted start
    FEATURE_ACP_CHATS = feature(Feature::AcpChats);
    FEATURE_QA_MENU = feature(Feature::QaMenu);
    FEATURE_SIDEBAR_HOVER_MENU = feature(Feature::SidebarHoverMenu);
    // keep-sorted end
    GENERAL_CHAT_LAYOUT = entry(Section::General, "general-chat-layout", "settings-general-chat-layout", "settings-general-chat-layout-description").only(Availability::AcpChats);
    GENERAL_CHAT_FULL_WIDTH = entry(Section::General, "general-chat-full-width", "settings-general-chat-full-width", "settings-general-chat-full-width-description").only(Availability::AcpChats);
    GENERAL_CHAT_WIDTH = entry(Section::General, "general-chat-width", "settings-general-chat-width", "settings-general-chat-width-description").only(Availability::LimitedChatWidth);
    GENERAL_CHAT_LAUNCHER = entry(Section::General, "general-chat-launcher", "settings-general-chat-launcher", "settings-general-chat-launcher-description").only(Availability::AcpChats);
    GENERAL_CHAT_GLIDE = entry(Section::General, "general-chat-glide", "settings-general-chat-glide", "settings-general-chat-glide-description").only(Availability::AcpChats);
    GENERAL_COPY_ON_SELECT = entry(Section::General, "general-copy-on-select", "settings-general-copy-on-select", "settings-general-copy-on-select-description");
    GENERAL_COPIED_TOAST = entry(Section::General, "general-copied-toast", "settings-general-copied-toast", "");
    GENERAL_CLIPBOARD_POSITION = entry(Section::General, "general-clipboard-position", "settings-general-clipboard-position", "");
    GENERAL_BROWSER_SKILL = entry(Section::General, "general-browser-skill", "settings-general-browser-skill", "settings-general-browser-skill-description");
    GENERAL_LAUNCH_AT_LOGIN = entry(Section::General, "general-launch-at-login", "settings-general-launch-at-login", "settings-general-launch-at-login-description").only(Availability::Windows);
    TABS_LAYOUT = entry(Section::Tabs, "tabs-layout", "settings-tabs-layout", "settings-tabs-layout-description");
    TABS_POSITION = entry(Section::Tabs, "tabs-position", "settings-tabs-position", "");
    TABS_HERDR_2 = entry(Section::Tabs, "tabs-herdr-2", "settings-tabs-herdr-2", "settings-tabs-herdr-2-description").only(Availability::Core);
    TABS_AGENT_TITLE = entry(Section::Tabs, "tabs-agent-title", "settings-tabs-agent-title", "settings-tabs-agent-title-description");
    TABS_HIDE_SINGLE = entry(Section::Tabs, "tabs-hide-single", "settings-tabs-hide-single", "");
    TABS_CONFIRM_TAB = entry(Section::Tabs, "tabs-confirm-tab", "settings-tabs-confirm-tab", "");
    TABS_CONFIRM_PANE = entry(Section::Tabs, "tabs-confirm-pane", "settings-tabs-confirm-pane", "");
    TABS_FOCUS_FOLLOWS_MOUSE = entry(Section::Tabs, "tabs-focus-follows-mouse", "settings-tabs-focus-follows-mouse", "settings-tabs-focus-follows-mouse-description");
    TABS_PANE_HISTORY = entry(Section::Tabs, "tabs-pane-history", "settings-tabs-pane-history", "settings-tabs-pane-history-description");
    NOTIF_DELIVERY = entry(Section::Notifications, "notif-delivery", "settings-notif-delivery", "settings-notif-delivery-description");
    NOTIF_FINISHED = entry(Section::Notifications, "notif-finished", "settings-notif-finished", "settings-notifications-finished-description");
    NOTIF_SHOW = entry(Section::Notifications, "notif-show", "settings-notif-show", "");
    NOTIF_DELAY = entry(Section::Notifications, "notif-delay", "settings-notif-delay", "settings-notifications-delay-description");
    NOTIF_POSITION = entry(Section::Notifications, "notif-position", "settings-notif-position", "settings-notifications-position-description");
    NOTIF_SOUND = entry(Section::Notifications, "notif-sound", "settings-notif-sound", "settings-notifications-sound-description");
    NOTIF_VOLUME = entry(Section::Notifications, "notif-volume", "settings-notif-volume", "settings-notif-volume-description");
    INTEGRATIONS_SEARCH = entry(Section::Integrations, "integrations-search", "settings-integrations-search", "settings-integrations-search-description");
    CONFIG_LOCAL = entry(Section::Configuration, "config-local", "settings-config-local", "settings-config-local-description");
    CONFIG_SHARED = entry(Section::Configuration, "config-shared", "settings-config-shared", "settings-config-shared-description");
    CONFIG_RELOAD = entry(Section::Configuration, "config-reload", "settings-config-reload", "settings-config-reload-description");
}

/// Indices into [`CATALOG`] of the entries whose title, description or page
/// name contains `query`, ignoring case, in catalog order. Each is matched in
/// the current locale and in English, as the palette matches commands.
pub(super) fn search(query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    CATALOG
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            [entry.title, entry.description, entry.section.message()]
                .iter()
                .filter(|id| !id.is_empty())
                .flat_map(|id| [i18n::message(id, &[]), i18n::english(id, &[])])
                .any(|text| text.to_lowercase().contains(&query))
        })
        .map(|(index, _)| index)
        .collect()
}
