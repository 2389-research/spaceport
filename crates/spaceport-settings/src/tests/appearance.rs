use super::*;

fn pair() -> Config {
    Config {
        theme: "light:Catppuccin Latte,dark:Nord".into(),
        ..Default::default()
    }
}

#[test]
fn an_appearance_carries_the_theme_its_config_named() -> anyhow::Result<()> {
    let dark = Appearance::of(pair(), false)?;
    assert_eq!(Some(dark.theme), Theme::builtin("Nord"));
    assert_eq!(dark.config.theme, "light:Catppuccin Latte,dark:Nord");
    assert_eq!(
        Theme::builtin("Catppuccin Latte"),
        Some(Appearance::of(pair(), true)?.theme)
    );
    Ok(())
}

#[test]
fn a_theme_that_names_no_file_resolves_to_nothing() {
    let config = Config {
        theme: "../invalid".into(),
        ..Default::default()
    };
    assert!(Appearance::of(config, false).is_err());
}

#[test]
fn both_sides_resolve_before_the_first_frame() -> anyhow::Result<()> {
    let (sides, error) = ThemeSides::of(Appearance::of(pair(), false)?);
    assert!(error.is_none());
    assert_eq!(
        Theme::builtin("Nord"),
        Some(sides.clone().select(false).theme)
    );
    assert_eq!(
        Theme::builtin("Catppuccin Latte"),
        Some(sides.select(true).theme)
    );
    // A theme naming one side shows it either way, and reads no second file.
    let (single, error) = ThemeSides::of(Appearance::of(
        Config {
            theme: "Nord".into(),
            ..Default::default()
        },
        false,
    )?);
    assert!(error.is_none());
    assert_eq!(Theme::builtin("Nord"), Some(single.select(true).theme));
    Ok(())
}

#[test]
fn a_failed_light_theme_returns_its_error_and_keeps_the_dark_theme() -> anyhow::Result<()> {
    let config = Config {
        theme: "light:../invalid,dark:Nord".into(),
        ..Default::default()
    };
    let (sides, error) = ThemeSides::of(Appearance::of(config, false)?);
    assert!(matches!(error, Some(Error::InvalidThemePath)));
    assert_eq!(Some(sides.select(true).theme), Theme::builtin("Nord"));
    Ok(())
}
