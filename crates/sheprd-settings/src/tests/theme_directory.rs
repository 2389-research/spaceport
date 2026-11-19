use super::*;

#[test]
fn named_app_themes_use_spaceports_config_root() -> anyhow::Result<()> {
    let root = TempDirectory::new()?;
    let app = root.0.join("spaceport/themes");
    let upstream = root.0.join("herdr/themes");
    fs::create_dir_all(&app)?;
    fs::create_dir_all(&upstream)?;
    fs::write(app.join("spaceport-fixture-theme"), "background=112233")?;
    fs::write(upstream.join("upstream-fixture-theme"), "background=445566")?;

    let directories = theme::theme_directories_at(&root.0);
    assert_eq!(directories.first(), Some(&app));
    assert!(!directories.contains(&upstream));
    let names = Config::default().available_themes_in(&directories[..1])?;
    assert!(names.contains(&"spaceport-fixture-theme".to_owned()));
    assert!(!names.contains(&"upstream-fixture-theme".to_owned()));
    Ok(())
}
