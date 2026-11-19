// ABOUTME: Checks localization of settings and theme error variants.
// ABOUTME: Keeps English diagnostics aligned with their typed error displays.
use super::*;
use crate::config::{Error, ThemeParseError, tab_layout};
use std::path::PathBuf;

fn parse_int() -> std::num::ParseIntError {
    "x".parse::<u8>().unwrap_err()
}

#[test]
fn every_config_variant_has_a_message() {
    assert_localized(&[
        Error::DeviceTheme {
            device: "ssh-build-box".into(),
            source: Box::new(Error::EmptyTheme),
        },
        Error::EmptyFontFallback("font"),
        Error::EmptyFontFamily("font"),
        Error::EmptyTheme,
        Error::InvalidChatMirror,
        Error::InvalidChatWidth,
        Error::InvalidDeviceId("nope".into()),
        Error::InvalidDevicesTable,
        Error::InvalidFavoriteThemes,
        Error::InvalidFontSize("zen"),
        Error::InvalidFontWeight("font"),
        Error::InvalidHostColor {
            host: "build-box".into(),
            value: "blue".into(),
        },
        Error::InvalidLineHeight("font"),
        Error::InvalidSidebarGap,
        Error::InvalidSidebarMetric {
            key: "row_gap",
            max: 64.0,
        },
        Error::InvalidSidebarMetric {
            key: "indent",
            max: 12.5,
        },
        Error::InvalidSoundVolume,
        Error::InvalidTerminalPadding("padding"),
        Error::InvalidThemePair,
        Error::InvalidThemePath,
        Error::InvalidUsageTable,
        Error::InvalidZenFontSize,
        Error::InvalidZenWidth,
        Error::KeyNotTable {
            key: "sidebar.rows".into(),
        },
        Error::MigrationConflict {
            original: PathBuf::from("/home/me/.config/herdr/config-gpui.toml"),
            local: PathBuf::from("/home/me/.config/herdr/config-gpui.local.toml"),
        },
        Error::PreferencesNotObject,
        Error::RemovedChangeCounts,
        Error::RemovedGitHub,
        Error::ThemeLine {
            line: 12,
            key: "background".into(),
            source: ThemeParseError::InvalidColor,
        },
        Error::ThemeNotFound {
            name: "Dracula".into(),
            directories: vec![PathBuf::from("/a/themes"), PathBuf::from("/b/themes")],
        },
        Error::TooManyDevices(64),
        Error::TooManyFontFallbacks("font"),
        Error::UnknownKeys(vec!["colour".into()]),
        Error::UnknownKeys(vec!["colour".into(), "fnot.size".into()]),
        Error::UnknownLayout("columns".into()),
    ]);
}

#[test]
fn every_theme_parse_variant_has_a_message() {
    assert_localized(&[
        ThemeParseError::InvalidColor,
        ThemeParseError::InvalidHex(parse_int()),
        ThemeParseError::MissingPaletteColor,
        ThemeParseError::InvalidPaletteIndex(parse_int()),
        ThemeParseError::PaletteIndexOutOfRange,
    ]);
}

#[test]
fn every_ssh_hosts_variant_has_a_message() {
    assert_localized(&[
        sheprd_settings::ssh_hosts::Error::InvalidTarget("-oProxyCommand=x".into()),
        sheprd_settings::ssh_hosts::Error::TooMany(256),
    ]);
}

#[test]
fn every_tab_layout_variant_has_a_message() {
    assert_localized(&[tab_layout::Error::Unknown("ribbons".into())]);
}
