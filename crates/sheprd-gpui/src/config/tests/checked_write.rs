//! The check the Settings window's writer makes before it replaces the
//! overrides file: an edit may not break a file that loads.
use super::*;

#[test]
fn an_edit_that_breaks_a_loading_file_is_refused_with_its_cause() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    let before = "theme = 'Nord' # keep\n";
    fs::write(&path, before)?;
    for (edited, expected) in [
        (
            "theme = 'Nord' # keep\n[sidebar]\nindent = 49\n",
            (|error| {
                matches!(
                    error,
                    crate::Error::Settings(Error::InvalidSidebarMetric { key: "indent", .. })
                )
            }) as fn(&crate::Error) -> bool,
        ),
        ("theme = ''\n", |error| {
            matches!(error, crate::Error::Settings(Error::EmptyTheme))
        }),
        // A bound the parser itself holds comes back as the parser's error.
        ("[sound]\nvolume = 500\n", |error| {
            !matches!(error, crate::Error::Settings(_))
        }),
    ] {
        let error = Config::write_unless_broken(&path, before, edited)
            .err()
            .with_context(|| format!("{edited:?} was written"))?;
        assert!(expected(&error), "{edited:?}: {error:?}");
        assert_eq!(fs::read_to_string(&path)?, before, "{edited:?}");
    }
    Ok(())
}

#[test]
fn an_edit_that_loads_is_written() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    // An unknown key is reported when loading, not refused, so it is no
    // reason to refuse the write either.
    let edited = "future = 1\nconfirm_close_tab = false\n";
    Config::write_unless_broken(&path, "future = 1\n", edited)?;
    assert_eq!(fs::read_to_string(&path)?, edited);
    Ok(())
}

#[test]
fn a_file_the_loader_already_refuses_is_written_as_asked() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("local.toml");
    let before = "[sound]\nvolume = 500\n";
    assert!(Config::parse(before).is_err());
    // Still broken, by the same mistake: the edit beside it lands.
    let beside = "confirm_close_tab = false\n[sound]\nvolume = 500\n";
    Config::write_unless_broken(&path, before, beside)?;
    assert_eq!(fs::read_to_string(&path)?, beside);
    // And the repair lands.
    let repaired = "confirm_close_tab = false\n[sound]\nvolume = 50\n";
    Config::write_unless_broken(&path, beside, repaired)?;
    assert_eq!(fs::read_to_string(&path)?, repaired);
    assert_eq!(Config::parse(repaired)?.sound.volume, 50);
    Ok(())
}
