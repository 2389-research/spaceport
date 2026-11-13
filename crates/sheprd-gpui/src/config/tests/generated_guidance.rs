// ABOUTME: Checks guidance in the actual managed and local files from first launch.
// ABOUTME: Keeps Spaceport setup instructions separate from upstream app state.
use super::*;

#[test]
fn generated_guidance_names_the_actual_spaceport_files_and_command() -> anyhow::Result<()> {
    let root = TempDirectory::new()?;
    let managed = root.0.join("config.toml");
    Config::load_path(&managed, &root.0.join("absent-daemon.toml"))?;
    let defaults = fs::read_to_string(&managed)?;
    let local = fs::read_to_string(managed.with_extension("local.toml"))?;
    assert!(defaults.contains("# Managed Spaceport defaults"));
    assert!(defaults.contains("spaceport/config.toml"));
    assert!(defaults.contains("spaceport config set chat.max_width 0"));
    assert!(!defaults.contains("config-gpui"));
    assert!(!defaults.contains("Sheprd"));
    assert!(local.contains("inherit config.toml"));
    assert!(!local.contains("config-gpui"));
    Ok(())
}
