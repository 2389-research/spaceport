//! The number and dropdown fields the Settings pages show: the id each is
//! found by, the values it accepts, and the tables of choices it lists.
//!
//! A choice is here as its message, not as words: the app words every one.

use crate::catalog;
use crate::chat::MAX_WIDTH_RANGE;
use crate::fonts::{FONT_SIZE_RANGE, FontFace};
use crate::notifications::{ClipboardToastPosition, SoundConfig};
use crate::padding::{Axis, MAX_PADDING, Side};
use crate::sidebar_style::SidebarOverrides;
use spaceport_client::protocol::ToastHerdrPosition;
use std::ops::RangeInclusive;

/// Every number field the pages show; the selector is the control's id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumberField {
    ChatMaxWidth,
    Spacing(Spacing),
    FontSize(FontFace),
    ToastDelay,
    SoundVolume,
    TerminalPadding(Axis, Side),
}

impl NumberField {
    pub fn selector(self) -> &'static str {
        match self {
            Self::ChatMaxWidth => "general-chat-width-control",
            Self::Spacing(spacing) => spacing.control(),
            Self::FontSize(face) => size_control(face),
            Self::ToastDelay => "notif-delay-control",
            Self::SoundVolume => "notif-volume-control",
            Self::TerminalPadding(axis, side) => padding_ids(axis, side).1,
        }
    }

    /// The values the field accepts.
    pub fn range(self) -> RangeInclusive<f32> {
        match self {
            Self::ChatMaxWidth => {
                f32::from(*MAX_WIDTH_RANGE.start())..=f32::from(*MAX_WIDTH_RANGE.end())
            }
            Self::Spacing(spacing) => spacing.range(),
            Self::FontSize(_) => FONT_SIZE_RANGE,
            Self::ToastDelay => 0.0..=3600.0,
            Self::SoundVolume => 0.0..=f32::from(SoundConfig::MAX_VOLUME),
            Self::TerminalPadding(..) => 0.0..=MAX_PADDING,
        }
    }
}

/// Every dropdown the pages show; the selector is the button's id.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropdownField {
    Layout,
    ClipboardEnabled,
    ClipboardPosition,
    Language,
    ToastEnabled,
    ToastPosition,
}

impl DropdownField {
    pub const fn selector(self) -> &'static str {
        match self {
            Self::Layout => "sidebar-layout-control",
            Self::ClipboardEnabled => "general-copied-toast-control",
            Self::ClipboardPosition => "general-clipboard-position-control",
            Self::Language => "general-language-control",
            Self::ToastEnabled => "notif-show-control",
            Self::ToastPosition => "notif-position-control",
        }
    }
}

/// The four spacing numbers the Sidebar page edits. The old sidebar gap
/// became the terminal's left padding, on the Theme page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spacing {
    Indent,
    RowPadding,
    RowGap,
    HostGap,
}

impl Spacing {
    pub const ALL: [Self; 4] = [Self::Indent, Self::RowPadding, Self::RowGap, Self::HostGap];

    /// The catalog entry of the row.
    pub const fn entry(self) -> &'static catalog::Entry {
        match self {
            Self::Indent => &catalog::SIDEBAR_INDENT,
            Self::RowPadding => &catalog::SIDEBAR_ROW_PADDING,
            Self::RowGap => &catalog::SIDEBAR_ROW_GAP,
            Self::HostGap => &catalog::SIDEBAR_HOST_GAP,
        }
    }

    pub const fn control(self) -> &'static str {
        match self {
            Self::Indent => "sidebar-indent-control",
            Self::RowPadding => "sidebar-row-padding-control",
            Self::RowGap => "sidebar-row-gap-control",
            Self::HostGap => "sidebar-host-gap-control",
        }
    }

    /// The id of the button that clears this number's override.
    pub const fn reset(self) -> &'static str {
        match self {
            Self::Indent => "sidebar-indent-reset",
            Self::RowPadding => "sidebar-row-padding-reset",
            Self::RowGap => "sidebar-row-gap-reset",
            Self::HostGap => "sidebar-host-gap-reset",
        }
    }

    /// The `[sidebar]` key the parser bands.
    const fn band(self) -> &'static str {
        match self {
            Self::Indent => "indent",
            Self::RowPadding => "row_padding",
            Self::RowGap => "gap",
            Self::HostGap => "host_gap",
        }
    }

    pub fn range(self) -> RangeInclusive<f32> {
        let key = self.band();
        let max = SidebarOverrides::BANDS
            .iter()
            .find_map(|(name, max)| (*name == key).then_some(*max))
            .unwrap_or(0.);
        0.0..=max
    }

    /// The `[sidebar]` slot this number is written to.
    pub fn slot(self, overrides: &mut SidebarOverrides) -> &mut Option<f32> {
        match self {
            Self::Indent => &mut overrides.indent,
            Self::RowPadding => &mut overrides.row_padding,
            Self::RowGap => &mut overrides.gap,
            Self::HostGap => &mut overrides.host_gap,
        }
    }
}

/// The message naming a font face's row, which the catalog and the font
/// controls both show.
pub const fn face_message(face: FontFace) -> &'static str {
    match face {
        FontFace::Terminal => "settings-fonts-terminal",
        FontFace::Sidebar => "settings-fonts-sidebar",
        FontFace::Tabs => "settings-fonts-tabs",
        FontFace::Ui => "settings-fonts-ui",
    }
}

/// The id of the size field for `face`.
pub const fn size_control(face: FontFace) -> &'static str {
    match face {
        FontFace::Terminal => "fonts-terminal-size-control",
        FontFace::Sidebar => "fonts-sidebar-size-control",
        FontFace::Tabs => "fonts-tabs-size-control",
        FontFace::Ui => "fonts-ui-size-control",
    }
}

/// The label row and the field id of one side's terminal padding.
pub fn padding_ids(axis: Axis, side: Side) -> (&'static catalog::Entry, &'static str) {
    match (axis, side) {
        (Axis::X, Side::Start) => (&catalog::THEME_PADDING_LEFT, "terminal-padding-left"),
        (Axis::X, Side::End) => (&catalog::THEME_PADDING_RIGHT, "terminal-padding-right"),
        (Axis::Y, Side::Start) => (&catalog::THEME_PADDING_TOP, "terminal-padding-top"),
        (Axis::Y, Side::End) => (&catalog::THEME_PADDING_BOTTOM, "terminal-padding-bottom"),
    }
}

/// The clipboard notification's positions, each with its message.
pub const CLIPBOARD_POSITIONS: [(ClipboardToastPosition, &str); 6] = {
    use ClipboardToastPosition as P;
    [
        (P::TopLeft, "settings-position-top-left"),
        (P::TopCenter, "settings-position-top-center"),
        (P::TopRight, "settings-position-top-right"),
        (P::BottomLeft, "settings-position-bottom-left"),
        (P::BottomCenter, "settings-position-bottom-center"),
        (P::BottomRight, "settings-position-bottom-right"),
    ]
};

/// Herdr's toast positions, each with its message.
pub const TOAST_POSITIONS: [(ToastHerdrPosition, &str); 4] = {
    use ToastHerdrPosition as P;
    [
        (P::TopLeft, "settings-position-top-left"),
        (P::TopRight, "settings-position-top-right"),
        (P::BottomLeft, "settings-position-bottom-left"),
        (P::BottomRight, "settings-position-bottom-right"),
    ]
};

/// The message naming `position` among `positions`, or none when it is missing.
pub fn position_message<T: PartialEq>(
    positions: &[(T, &'static str)],
    position: &T,
) -> Option<&'static str> {
    positions
        .iter()
        .find(|(p, _)| p == position)
        .map(|(_, id)| *id)
}

/// The message naming "On" or "Off", for a boolean override dropdown.
pub const fn on_off_message(value: bool) -> &'static str {
    if value { "settings-on" } else { "settings-off" }
}

/// The messages of a boolean override dropdown's choices, on first.
pub const fn on_off_messages() -> [&'static str; 2] {
    [on_off_message(true), on_off_message(false)]
}
