use super::*;
use crate::config::{bold_font_weight, font_weight::MissingWeight, settings_weight, weight};

#[test]
fn every_face_draws_regular_text_until_a_weight_is_configured() -> anyhow::Result<()> {
    for config in [
        Config::default(),
        Config::parse("")?,
        Config::parse(DEFAULT_CONFIG)?,
    ] {
        for font in [&config.sidebar, &config.tabs, &config.terminal, &config.ui] {
            assert_eq!(font.weight, Weight::NORMAL);
            assert_eq!(settings_weight(font.font().weight), Weight::NORMAL);
        }
    }
    Ok(())
}

#[test]
fn terminal_weight_takes_css_names_and_numbers() -> anyhow::Result<()> {
    for (value, configured) in [
        ("'medium'", Weight::MEDIUM),
        ("'Medium'", Weight::MEDIUM),
        ("'semi-bold'", Weight::SEMIBOLD),
        ("'Extra Light'", Weight::EXTRA_LIGHT),
        ("'regular'", Weight::NORMAL),
        ("'heavy'", Weight::BLACK),
        ("100", Weight::THIN),
        ("450", Weight(450.0)),
        ("900", Weight::BLACK),
    ] {
        let config = Config::parse(&format!("[terminal]\nweight = {value}"))?;
        assert_eq!(config.terminal.weight, configured, "weight = {value}");
        assert_eq!(
            settings_weight(config.terminal.font().weight),
            configured,
            "weight = {value}"
        );
        assert!(config.unknown_keys.is_empty(), "{:?}", config.unknown_keys);
    }
    Ok(())
}

#[test]
fn local_weight_merges_over_managed_defaults() -> anyhow::Result<()> {
    let config = Config::parse_layers(
        [DEFAULT_CONFIG, "[terminal]\nweight = 'medium'"],
        &Daemon::default(),
    )?;
    assert_eq!(config.terminal.size, 14.0);
    assert_eq!(config.terminal.weight, Weight::MEDIUM);
    Ok(())
}

#[test]
fn rejects_unknown_names_and_out_of_range_weights() {
    for value in [
        "'mediumish'",
        "''",
        "99",
        "901",
        "0",
        "-400",
        "500.0",
        "true",
        "['medium']",
    ] {
        assert!(
            matches!(
                Config::parse(&format!("[terminal]\nweight = {value}")),
                Err(crate::Error::Settings(Error::InvalidFontWeight("terminal")))
            ),
            "accepted weight = {value}"
        );
    }
}

#[test]
fn weight_outside_terminal_is_reported_and_ignored() -> anyhow::Result<()> {
    // Invalid too: an ignored key is never validated.
    let config =
        Config::parse("[sidebar]\nweight = 'medium'\n[tabs]\nweight = 500\n[ui]\nweight = 'nope'")?;
    assert_eq!(
        config.unknown_keys,
        ["sidebar.weight", "tabs.weight", "ui.weight"]
    );
    for font in [&config.sidebar, &config.tabs, &config.ui] {
        assert_eq!(font.weight, Weight::NORMAL);
    }
    Ok(())
}

#[test]
fn bold_cells_stay_heavier_than_the_configured_weight() {
    for (base, bold) in [
        (Weight::THIN, Weight::BOLD),
        (Weight::LIGHT, Weight::BOLD),
        (Weight::NORMAL, Weight::BOLD),
        (Weight::MEDIUM, Weight::EXTRA_BOLD),
        (Weight::SEMIBOLD, Weight::BLACK),
        (Weight::BOLD, Weight::BLACK),
        (Weight::BLACK, Weight::BLACK),
    ] {
        assert_eq!(
            settings_weight(bold_font_weight(weight(base))),
            bold,
            "base {base:?}"
        );
    }
}

fn installed() -> Vec<String> {
    ["Menlo", "JetBrains Mono"].map(str::to_owned).into()
}

#[test]
fn a_weight_the_family_lacks_is_reported() -> anyhow::Result<()> {
    let mut config = Config::parse("[terminal]\nfamily = 'Menlo'\nweight = 'medium'")?;
    config.resolve_fonts_with(installed, |_| false);
    config.check_font_weight(|family, configured| {
        assert_eq!((family, configured), ("Menlo", Weight::MEDIUM));
        false
    });
    assert_eq!(
        config.missing_weight,
        Some(MissingWeight {
            family: "Menlo".into(),
            weight: Weight::MEDIUM,
        })
    );
    assert_eq!(
        plain(config.diagnostic()).as_deref(),
        Some("\"Menlo\" has no medium (500) face; using its nearest weight")
    );

    // A number without a CSS name prints alone.
    let mut config = Config::parse("[terminal]\nfamily = 'Menlo'\nweight = 450")?;
    config.check_font_weight(|_, _| false);
    assert_eq!(
        plain(config.diagnostic()).as_deref(),
        Some("\"Menlo\" has no 450 face; using its nearest weight")
    );
    Ok(())
}

#[test]
fn installed_weights_regular_text_and_missing_families_are_not_reported() -> anyhow::Result<()> {
    let mut config = Config::parse("[terminal]\nfamily = 'JetBrains Mono'\nweight = 'medium'")?;
    config.resolve_fonts_with(installed, |_| false);
    config.check_font_weight(|_, _| true);
    assert_eq!(config.missing_weight, None);
    assert_eq!(config.diagnostic(), None);

    // Regular text needs no check: every family draws something there.
    let mut config = Config::parse("[terminal]\nfamily = 'JetBrains Mono'")?;
    config.check_font_weight(|_, _| panic!("regular weight was checked"));
    assert_eq!(config.missing_weight, None);

    // A family that is not installed is reported once, as missing.
    let mut config = Config::parse("[terminal]\nfamily = 'Nope Mono'\nweight = 'medium'")?;
    config.resolve_fonts_with(installed, |_| false);
    config.check_font_weight(|_, _| panic!("a missing family was checked"));
    assert_eq!(config.missing_weight, None);
    assert_eq!(
        plain(config.diagnostic()).as_deref(),
        Some("Configured fonts not installed: \"Nope Mono\"")
    );
    Ok(())
}

/// Menlo ships regular and bold faces only, and every macOS release has both.
#[cfg(target_os = "macos")]
#[test]
fn the_platform_lists_a_family_by_weight() {
    use crate::config::font_weight::platform_has_weight;
    assert!(platform_has_weight("Menlo", Weight::NORMAL));
    assert!(platform_has_weight("Menlo", Weight::BOLD));
    assert!(!platform_has_weight("Menlo", Weight::MEDIUM));
    assert!(!platform_has_weight("Menlo", Weight::THIN));
    // Helvetica Neue ships a medium face with every macOS release.
    assert!(platform_has_weight("Helvetica Neue", Weight::MEDIUM));
}
