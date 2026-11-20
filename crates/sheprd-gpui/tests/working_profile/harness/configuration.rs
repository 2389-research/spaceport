// ABOUTME: Checks profile configuration through the app's real migration and loading code.
// ABOUTME: Keeps common fixture settings when the mixed workload enables Core.
use super::*;
use sheprd_settings::{Config, SkillInstall};

fn configured(core_enabled: bool) -> Result<Config> {
    let mut fixture = Fixture::prepare(core_enabled)?;
    let directory = fixture.sandbox()?.dir.join("config/spaceport");
    let path = directory.join("config.toml");
    let (lock, _) = Config::prepare_files(&path)?;
    drop(lock);
    let config = Config::load_fixture(&path, &directory.join("view.toml"))?;
    assert!(!config.show_listening_ports);
    assert_eq!(config.agent_skill.install, SkillInstall::Off);
    assert_eq!(config.herdr_core_enabled, core_enabled);
    fixture.sandbox.take();
    Ok(config)
}

#[test]
fn tui_profile_keeps_common_settings_after_real_file_maintenance() -> Result<()> {
    configured(false).map(|_| ())
}

#[test]
fn mixed_profile_keeps_common_settings_after_real_file_maintenance() -> Result<()> {
    configured(true).map(|_| ())
}
