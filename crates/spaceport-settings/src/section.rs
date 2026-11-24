//! The Settings pages and the groups the section list shows them under.
//!
//! Each page and group names its own message; the app words it, as it words
//! every row the catalog declares.

/// One Settings page. `ALL` is the section list's order: General alone at the
/// top, then group by group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Section {
    General,
    #[default]
    Theme,
    Sidebar,
    Indicators,
    Fonts,
    StatusBar,
    Tabs,
    Notifications,
    Integrations,
    Configuration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Appearance,
    Behaviour,
    Connect,
    Advanced,
}

impl Group {
    pub const ALL: [Self; 4] = [
        Self::Appearance,
        Self::Behaviour,
        Self::Connect,
        Self::Advanced,
    ];

    /// The message naming the group.
    pub const fn message(self) -> &'static str {
        match self {
            Self::Appearance => "settings-group-appearance",
            Self::Behaviour => "settings-group-behaviour",
            Self::Connect => "settings-group-connect",
            Self::Advanced => "settings-group-advanced",
        }
    }

    /// The group label's selector, the same in every language.
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::Behaviour => "behaviour",
            Self::Connect => "connect",
            Self::Advanced => "advanced",
        }
    }
}

impl Section {
    /// The section list's order: General first, outside every group, as the
    /// catch-all people look in when they do not know where a setting lives;
    /// then each group's pages together, groups in [`Group::ALL`] order. Users
    /// see this order, so it is not sorted.
    pub const ALL: &[Self] = &[
        Self::General,
        Self::Theme,
        Self::Sidebar,
        Self::Indicators,
        Self::Fonts,
        Self::StatusBar,
        Self::Tabs,
        Self::Notifications,
        Self::Integrations,
        Self::Configuration,
    ];

    /// The message naming the page, which search also matches.
    pub const fn message(self) -> &'static str {
        match self {
            Self::Theme => "settings-section-theme",
            Self::Sidebar => "settings-section-sidebar",
            Self::Indicators => "settings-section-indicators",
            Self::Fonts => "settings-section-fonts",
            Self::StatusBar => "settings-section-status-bar",
            Self::General => "settings-section-general",
            Self::Tabs => "settings-section-tabs-and-panes",
            Self::Notifications => "settings-section-notifications",
            Self::Integrations => "settings-section-integrations",
            Self::Configuration => "settings-section-configuration",
        }
    }

    /// The message introducing the page, or none for a page that needs none.
    pub const fn lead(self) -> Option<&'static str> {
        Some(match self {
            Self::Theme => "settings-lead-theme",
            Self::Sidebar => "settings-lead-sidebar",
            Self::Indicators => "settings-lead-indicators",
            Self::Fonts => "settings-lead-fonts",
            Self::StatusBar => "settings-lead-status-bar",
            Self::Integrations => "settings-lead-integrations",
            Self::Configuration => "settings-lead-configuration",
            Self::General | Self::Tabs | Self::Notifications => return None,
        })
    }

    /// The group the section list shows this page under; `None` for pages
    /// listed above every group label.
    pub const fn group(self) -> Option<Group> {
        match self {
            Self::General => None,
            Self::Theme | Self::Sidebar | Self::Indicators | Self::Fonts | Self::StatusBar => {
                Some(Group::Appearance)
            }
            Self::Tabs | Self::Notifications => Some(Group::Behaviour),
            Self::Integrations => Some(Group::Connect),
            Self::Configuration => Some(Group::Advanced),
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::Theme => "icons/theme.svg",
            Self::Sidebar => "icons/panel-left.svg",
            Self::Indicators => "icons/pulse.svg",
            Self::Fonts => "icons/pencil.svg",
            Self::StatusBar => "icons/status-bar.svg",
            Self::General => "icons/settings.svg",
            Self::Tabs => "icons/tabs.svg",
            Self::Notifications => "icons/bell.svg",
            Self::Integrations => "icons/agent-generic.svg",
            Self::Configuration => "icons/file.svg",
        }
    }

    /// The page's selector, the same in every language: its message ID.
    pub fn selector(self) -> String {
        self.message().to_owned()
    }

    /// The section's index among the scrolled section list's children, which
    /// are the ungrouped pages, then each group's label followed by its pages.
    pub fn nav_item(self) -> usize {
        let mut item = 0;
        for group in std::iter::once(None).chain(Group::ALL.map(Some)) {
            if group.is_some() {
                item += 1;
            }
            for section in Self::ALL.iter().filter(|section| section.group() == group) {
                if *section == self {
                    return item;
                }
                item += 1;
            }
        }
        item
    }
}
