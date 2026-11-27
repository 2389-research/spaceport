// ABOUTME: Reads Spaceport's settings from the settings crate and words them for the app.
// ABOUTME: Keeps the keymap, the diagnostic, the SSH hook and the watcher with the app.
//! What the app itself does with settings: it builds its keymap from the raw
//! `[keybindings]` and `[pane_keys]` tables, words the diagnostic in the
//! reader's language, keeps `herdr-client`'s unverified-host list in step, and
//! watches the files.
//!
//! The values and their loading live in `spaceport-settings`; every name below is
//! re-exported so the app keeps one path for them.
// i18n: migrated
use crate::i18n::fl;
use spaceport_keymap::Keymap;
use spaceport_keymap::daemon::DaemonKeys;

pub(crate) mod ssh_hosts;
pub(crate) mod watch;

// The settings crate's names, under the path the app has always used. The
// list is temporary: each commit of the settings extraction deletes the
// aliases it made unnecessary, and nothing new belongs here. `pub` rather
// than `pub(crate)` so a name only the app's tests read is not reported
// unused in a plain build.
pub use spaceport_settings::{
    AgentLayout, AgentsPlacement, Axis, Backdrop, BellConfig, ChatLauncher, ChatLayoutMode,
    ClipboardToast, ClipboardToastPosition, Config, DeviceFooter, Error, FONT_SIZE_RANGE,
    FONT_SIZE_STEP, Feature, FontConfig, FontFace, KeybindingSource, LayoutMode, Material,
    NotificationConfig, NotificationDelivery, OptionAsAlt, OptionKeys, Padding, SelectMode, Side,
    SkillInstall, SoundConfig, SpaceLayout, StatusIcons, TabLayoutMode, ThemeName, Weight,
    ZenWidth,
};
// A density or style only the layout tests name.
#[cfg(any(test, feature = "integration-test"))]
pub use spaceport_settings::{Density, Style};
// The token kind only the pane-name layout tests name; the rows themselves are
// read in `spaceport-sidebar`.
#[cfg(test)]
pub use spaceport_settings::AgentToken;
pub use spaceport_settings::{
    FOLLOW_HERDR, Layer, SshConfig, Theme, Worktrees, backdrop, chat, corners, favorite_themes,
    font_weight, home, mix, parse_value, preferences, status_bar, tab_layout, view_settings_path,
};

/// Pads `element`, so the bounds its children are laid out in exclude the
/// padding.
pub(crate) fn pad<E: gpui::Styled>(element: E, padding: Padding) -> E {
    element
        .pl(gpui::px(padding.x.start()))
        .pr(gpui::px(padding.x.end()))
        .pt(gpui::px(padding.y.start()))
        .pb(gpui::px(padding.y.end()))
}

/// The shaping font for `face`, from `spaceport_widgets::fonts`, at the path the
/// app has always named it by.
pub(crate) use spaceport_widgets::fonts::font_for;

/// A configured weight as the renderer's.
pub(crate) fn weight(weight: Weight) -> gpui::FontWeight {
    gpui::FontWeight(weight.0)
}

/// The renderer's modifiers as a settings value.
pub(crate) fn modifiers(held: gpui::Modifiers) -> spaceport_settings::Modifiers {
    spaceport_settings::Modifiers {
        alt: held.alt,
        control: held.control,
        function: held.function,
        platform: held.platform,
        shift: held.shift,
    }
}

/// The renderer's weight as a configured one.
pub(crate) fn settings_weight(weight: gpui::FontWeight) -> Weight {
    Weight(weight.0)
}

/// The weight bold cells draw at: a fixed step heavier than `weight`, never
/// lighter than bold, so bold stays visible over a medium or light base.
pub(crate) fn bold_font_weight(weight: gpui::FontWeight) -> gpui::FontWeight {
    self::weight(font_weight::bold(settings_weight(weight)))
}

/// A settings value as the app titles it, in the reader's language. Settings
/// carries the message it names; choosing the words is the app's.
pub(crate) trait Label {
    fn label(self) -> String;
}

impl Label for ChatLauncher {
    fn label(self) -> String {
        crate::i18n::message(self.message(), &[])
    }
}

impl Label for ChatLayoutMode {
    fn label(self) -> String {
        crate::i18n::message(self.message(), &[])
    }
}

impl Label for LayoutMode {
    fn label(self) -> String {
        crate::i18n::message(self.message(), &[])
    }
}

impl Label for TabLayoutMode {
    fn label(self) -> String {
        crate::i18n::message(self.message(), &[])
    }
}

/// The keymap the app's keys and labels follow: the private view settings'
/// `[keys]` with this config's `[keybindings]` and `[pane_keys]` layered over
/// them. Built once per load, beside the settings it came from.
pub(crate) fn keymap(config: &Config) -> spaceport_keymap::Result<Keymap> {
    let keys = DaemonKeys::from_table(config.view_keys.as_ref());
    Keymap::with_overrides(&config.keybinding_overrides, &config.pane_keys, &keys)
}

/// Warnings naming the keys this build ignored and the configured fonts that
/// are not installed, one line each, if any. Keys and families come from the
/// user's file, so each loses the characters untrusted text does before it
/// joins a message.
pub(crate) fn diagnostic(config: &Config) -> Option<String> {
    const LISTED: usize = 5;
    const MAX_ITEM_CHARS: usize = 80;
    let list = |items: &[String]| {
        let listed: Vec<_> = items
            .iter()
            .take(LISTED)
            .map(|item| spaceport_client::text::safe_text(item, MAX_ITEM_CHARS))
            .collect();
        (listed.join(", "), items.len().saturating_sub(LISTED))
    };
    let mut lines = Vec::new();
    if !config.unknown_keys.is_empty() {
        let (keys, more) = list(&config.unknown_keys);
        lines.push(fl!(
            "config-diagnostic-unknown-keys",
            keys = keys,
            more = more
        ));
    }
    if config.layout.legacy_sidebar_gap.is_some() {
        lines.push(fl!("config-diagnostic-sidebar-gap"));
    }
    if !config.missing_fonts.is_empty() {
        let fonts: Vec<String> = config
            .missing_fonts
            .iter()
            .map(|family| format!("\"{family}\""))
            .collect();
        let (fonts, more) = list(&fonts);
        lines.push(fl!(
            "config-diagnostic-missing-fonts",
            fonts = fonts,
            more = more
        ));
    }
    if let Some(missing) = &config.missing_weight {
        lines.push(fl!(
            "config-diagnostic-missing-weight",
            family = spaceport_client::text::safe_text(&missing.family, MAX_ITEM_CHARS),
            weight = font_weight::describe(missing.weight)
        ));
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}
