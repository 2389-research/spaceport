use super::*;
use crate::Binding;
use std::cell::Cell;

/// What a caller's semantic check returns. The keymap's resolver lives in a
/// crate that depends on this one, so these tests stand in for it; the
/// app's tests run the real resolver through the same entry point.
#[derive(Debug, thiserror::Error)]
enum Check {
    #[error("rejected")]
    Rejected,
    #[error(transparent)]
    Settings(#[from] Error),
}

struct Files {
    _temp: tempfile::TempDir,
    managed: PathBuf,
    local: PathBuf,
    view: PathBuf,
}

fn files(local: Option<&str>) -> anyhow::Result<Files> {
    let temp = tempfile::tempdir()?;
    let managed = temp.path().join("config.toml");
    let local_path = temp.path().join("config.local.toml");
    if let Some(text) = local {
        fs::write(&local_path, text)?;
    }
    Ok(Files {
        managed,
        local: local_path,
        view: temp.path().join("view.toml"),
        _temp: temp,
    })
}

fn save(files: &Files, name: &str, binding: Option<Binding>) -> Result<(), Check> {
    Config::save_keybinding_at(&files.managed, &files.view, name, binding, |_| Ok(()))
}

#[test]
fn edit_unbind_and_reset_preserve_other_overrides_and_comments() -> anyhow::Result<()> {
    let files = files(Some(
        "# personal\ntheme = 'Nord'\n[keybindings]\nnew_tab = 'alt-t' # tab\nquit = 'alt-q'\n",
    ))?;
    let many = Binding::Many(vec!["ctrl-shift-t".into(), "alt-t".into()]);
    save(&files, "new_tab", Some(many.clone()))?;
    let text = fs::read_to_string(&files.local)?;
    assert!(text.contains("# personal") && text.contains("# tab"));
    let config = Config::load_fixture(&files.managed, &files.view)?;
    assert_eq!(config.theme, "Nord");
    assert_eq!(config.keybinding_overrides.get("new_tab"), Some(&many));
    assert_eq!(
        config.keybinding_overrides.get("quit"),
        Some(&Binding::One("alt-q".into()))
    );
    // An empty list unbinds; reset removes the override entirely.
    save(&files, "new_tab", Some(Binding::Many(vec![])))?;
    let config = Config::load_fixture(&files.managed, &files.view)?;
    assert_eq!(
        config.keybinding_overrides.get("new_tab"),
        Some(&Binding::Many(vec![]))
    );
    save(&files, "new_tab", None)?;
    let config = Config::load_fixture(&files.managed, &files.view)?;
    assert!(!config.keybinding_overrides.contains_key("new_tab"));
    assert!(config.keybinding_overrides.contains_key("quit"));
    Ok(())
}

#[test]
fn validation_sees_the_merged_file_and_a_rejection_never_writes() -> anyhow::Result<()> {
    // Another window saved `quit` after this caller's snapshot was taken.
    let original = "[keybindings]\nquit = 'alt-q'\n";
    let files = files(Some(original))?;
    let checked = Cell::new(false);
    let result = Config::save_keybinding_at(
        &files.managed,
        &files.view,
        "new_tab",
        Some(Binding::One("alt-t".into())),
        |candidate| {
            checked.set(true);
            assert_eq!(
                candidate.keybinding_overrides.get("quit"),
                Some(&Binding::One("alt-q".into()))
            );
            assert_eq!(
                candidate.keybinding_overrides.get("new_tab"),
                Some(&Binding::One("alt-t".into()))
            );
            Err(Check::Rejected)
        },
    );
    assert!(checked.get());
    assert!(matches!(result, Err(Check::Rejected)), "{result:?}");
    assert_eq!(fs::read_to_string(&files.local)?, original);
    Ok(())
}

#[test]
fn files_reload_would_reject_are_never_replaced_or_validated() -> anyhow::Result<()> {
    // An unrelated bad setting and a broken file fail the reread itself.
    for original in ["theme = ''\n", "[broken"] {
        let files = files(Some(original))?;
        let result = Config::save_keybinding_at(
            &files.managed,
            &files.view,
            "new_tab",
            None,
            |_| -> Result<(), Check> { panic!("validated a file reload rejects") },
        );
        assert!(matches!(result, Err(Check::Settings(_))), "{result:?}");
        assert_eq!(fs::read_to_string(&files.local)?, original);
    }
    let original = "[keybindings]\nquit = 'alt-q'\n";
    let files = files(Some(original))?;
    let result = save(&files, "unknown", None);
    assert!(
        matches!(
            &result,
            Err(Check::Settings(Error::Path { source, .. }))
                if matches!(**source, Error::UnknownKeybinding(_))
        ),
        "{result:?}"
    );
    assert_eq!(fs::read_to_string(&files.local)?, original);
    Ok(())
}

#[test]
fn inline_tables_and_missing_files_support_shortcut_edits() -> anyhow::Result<()> {
    let files = files(None)?;
    save(&files, "new_tab", Some(Binding::One("alt-t".into())))?;
    assert_eq!(
        Config::load_fixture(&files.managed, &files.view)?
            .keybinding_overrides
            .get("new_tab"),
        Some(&Binding::One("alt-t".into()))
    );
    fs::write(
        &files.local,
        "keybindings = {new_tab = 'alt-t', quit = 'alt-q'}\n",
    )?;
    save(&files, "new_tab", None)?;
    let config = Config::load_fixture(&files.managed, &files.view)?;
    assert!(!config.keybinding_overrides.contains_key("new_tab"));
    assert!(config.keybinding_overrides.contains_key("quit"));
    Ok(())
}
