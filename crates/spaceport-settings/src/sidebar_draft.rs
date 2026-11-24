//! A live edit of the sidebar's style: which keys it changed, and the
//! preferences that write them.
//!
//! A draft keeps every key it touched for its whole life, even one dragged
//! back to where the file already has it, and only touches the keys it did:
//! a reload keeps every other key as the file read it.

use crate::{Config, Preference, SidebarStyle};
use std::collections::BTreeSet;

/// One `[sidebar]` key a draft can change.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SidebarKey {
    Indent,
    RowPadding,
    Gap,
    HostGap,
    Select,
    DeviceFooter,
    ActivityBar,
    Automations,
    Host(String),
}

impl SidebarKey {
    const FIXED: &[Self] = &[
        Self::Indent,
        Self::RowPadding,
        Self::Gap,
        Self::HostGap,
        Self::Select,
        Self::DeviceFooter,
        Self::ActivityBar,
        Self::Automations,
    ];

    /// Copies this key's value from `from` into `to`.
    fn copy(&self, from: &SidebarStyle, to: &mut SidebarStyle) {
        let (old, new) = (&from.overrides, &mut to.overrides);
        match self {
            Self::Indent => new.indent = old.indent,
            Self::RowPadding => new.row_padding = old.row_padding,
            Self::Gap => new.gap = old.gap,
            Self::HostGap => new.host_gap = old.host_gap,
            Self::Select => to.select = from.select,
            Self::DeviceFooter => to.device_footer = from.device_footer,
            Self::ActivityBar => to.activity_bar = from.activity_bar,
            Self::Automations => to.automations = from.automations,
            Self::Host(host) => match from.hosts.get(host) {
                Some(color) => {
                    to.hosts.insert(host.clone(), *color);
                }
                None => {
                    to.hosts.remove(host);
                }
            },
        }
    }

    /// The write that puts `style`'s value for this key in the file.
    fn preference(&self, style: &SidebarStyle) -> Preference {
        let overrides = &style.overrides;
        match self {
            Self::Indent => Preference::SidebarIndent(overrides.indent),
            Self::RowPadding => Preference::SidebarRowPadding(overrides.row_padding),
            Self::Gap => Preference::SidebarRowGap(overrides.gap),
            Self::HostGap => Preference::SidebarHostGap(overrides.host_gap),
            Self::Select => Preference::SidebarSelect(style.select),
            Self::DeviceFooter => Preference::SidebarDeviceFooter(style.device_footer),
            Self::ActivityBar => Preference::SidebarActivityBar(style.activity_bar),
            Self::Automations => Preference::SidebarAutomations(style.automations),
            Self::Host(host) => Preference::HostColor {
                host: host.clone(),
                color: style.hosts.get(host).copied(),
            },
        }
    }

    fn differs(&self, before: &SidebarStyle, after: &SidebarStyle) -> bool {
        let (old, new) = (&before.overrides, &after.overrides);
        match self {
            Self::Indent => old.indent != new.indent,
            Self::RowPadding => old.row_padding != new.row_padding,
            Self::Gap => old.gap != new.gap,
            Self::HostGap => old.host_gap != new.host_gap,
            Self::Select => before.select != after.select,
            Self::DeviceFooter => before.device_footer != after.device_footer,
            Self::ActivityBar => before.activity_bar != after.activity_bar,
            Self::Automations => before.automations != after.automations,
            Self::Host(host) => before.hosts.get(host) != after.hosts.get(host),
        }
    }
}

/// The keys a draft has changed since it started. Only these are written,
/// and a reload keeps every other key as the file has it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SidebarTouched(BTreeSet<SidebarKey>);

impl SidebarTouched {
    /// Every key whose value differs between `before` and `after`.
    pub fn between(before: &SidebarStyle, after: &SidebarStyle) -> Self {
        let hosts = before
            .hosts
            .keys()
            .chain(after.hosts.keys())
            .map(|host| SidebarKey::Host(host.clone()));
        Self(
            SidebarKey::FIXED
                .iter()
                .cloned()
                .chain(hosts)
                .filter(|key| key.differs(before, after))
                .collect(),
        )
    }

    pub fn extend(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }

    pub fn keys(&self) -> impl Iterator<Item = &SidebarKey> {
        self.0.iter()
    }
}

/// What the Sidebar page shows and every main window uses until it is saved.
#[derive(Clone, Debug, PartialEq)]
pub struct SidebarIntent {
    pub style: SidebarStyle,
    /// Filled in by the page from what the draft changed.
    pub touched: SidebarTouched,
}

impl SidebarIntent {
    pub fn of(config: &Config) -> Self {
        Self {
            style: config.sidebar_style.clone(),
            touched: SidebarTouched::default(),
        }
    }

    /// Every key, touched or not.
    pub fn apply(&self, config: &mut Config) {
        config.sidebar_style = self.style.clone();
    }

    /// Only the keys this draft changed, over whatever `config` holds.
    pub fn overlay(&self, config: &mut Config) {
        for key in self.touched.keys() {
            key.copy(&self.style, &mut config.sidebar_style);
        }
    }

    /// Takes every untouched key from a fresh load, so a later write and
    /// the windows see what the file says there.
    pub fn rebase(&mut self, loaded: &Config) {
        let mut style = loaded.sidebar_style.clone();
        for key in self.touched.keys() {
            key.copy(&self.style, &mut style);
        }
        self.style = style;
    }
}

/// Every touched key whose value differs from `saved`, in one save's order.
pub fn diff_preferences(saved: &Config, intent: &SidebarIntent) -> Vec<Preference> {
    intent
        .touched
        .keys()
        .filter(|key| key.differs(&saved.sidebar_style, &intent.style))
        .map(|key| key.preference(&intent.style))
        .collect()
}

/// Every touched key of `intent`, for a quit that cannot know what an
/// in-flight save left on disk: a revert to the loaded value must still land.
pub fn touched_preferences(intent: &SidebarIntent) -> Vec<Preference> {
    intent
        .touched
        .keys()
        .map(|key| key.preference(&intent.style))
        .collect()
}
