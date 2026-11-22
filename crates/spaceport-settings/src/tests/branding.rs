// ABOUTME: Checks the settings heading produced for a fresh Spaceport install.
// ABOUTME: Proves later loads preserve an existing overrides file byte-for-byte.
use super::*;

#[test]
fn first_launch_names_spaceport_without_rewriting_existing_overrides() -> anyhow::Result<()> {
    let temp = TempDirectory::new()?;
    let path = temp.0.join("view.toml");
    let daemon = temp.0.join("absent.toml");
    let local = path.with_extension("local.toml");
    let config = Config::load_path(&path, &daemon)?;
    let created = fs::read_to_string(&local)?;
    assert_eq!(
        created.lines().next(),
        Some("# Spaceport overrides. Saved changes reload automatically.")
    );
    assert_eq!(
        config.layout.mode,
        LayoutMode::new(Density::Comfortable, Style::Rounded)
    );
    let existing = "# Sheprd overrides. Personal settings.\nlayout = 'compact'\ntheme = 'Nord'\n";
    fs::write(&local, existing)?;
    assert_eq!(Config::load_path(&path, &daemon)?.theme, "Nord");
    assert_eq!(fs::read_to_string(&local)?, existing);
    Ok(())
}
