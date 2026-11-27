//! Every variant of this crate's failures words itself for a window.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::{BackdropProblem, Error, ThemeParseError, ssh_hosts, tab_layout};
use spaceport_i18n::{Locale, Localize, assertions::assert_localized, set_locale};
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

/// The settings a message names are the files this build reads, not the
/// inherited `config-gpui.toml` the app never looks for.
#[test]
fn a_message_names_the_settings_file_it_speaks_of() {
    set_locale(&Locale::FALLBACK);
    let cases = [(
        Error::UnknownKeys(vec!["colour".into()]),
        "spaceport/config.toml",
    )];
    for (error, file) in cases {
        let shown = error.localize();
        assert!(shown.contains(file), "{shown}");
        assert!(!shown.contains("config-gpui"), "{shown}");
    }
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
        ssh_hosts::Error::InvalidTarget("-oProxyCommand=x".into()),
        ssh_hosts::Error::TooMany(256),
    ]);
}

#[test]
fn every_tab_layout_variant_has_a_message() {
    assert_localized(&[tab_layout::Error::Unknown("ribbons".into())]);
}

#[test]
fn every_backdrop_problem_has_a_message() {
    assert_localized(&[
        BackdropProblem::EmptyImage,
        BackdropProblem::ImagePathEncoding,
        BackdropProblem::Opacity,
        BackdropProblem::RelativeImage,
    ]);
}
