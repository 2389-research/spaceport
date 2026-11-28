// ABOUTME: Checks old VS Code settings cannot activate a removed IDE route.
// ABOUTME: Keeps other settings usable beside stale IDE keys.
use super::*;

#[test]
fn old_ide_and_px0_settings_are_unknown() -> anyhow::Result<()> {
    let config = Config::parse(
        "show_device_icons = false\n[code]\nmode = 'start'\nlicense_accepted = true\n[keybindings]\nopen_code = 'cmd-alt-c'\n[px0]\nurl = 'http://localhost:7777/'",
    )?;
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("code"))
    );
    assert!(
        config
            .unknown_keys
            .contains(&"keybindings.open_code".into())
    );
    assert!(!config.show_device_icons);
    assert!(config.unknown_keys.iter().any(|key| key.starts_with("px0")));
    Ok(())
}
