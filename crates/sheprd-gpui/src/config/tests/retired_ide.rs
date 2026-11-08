// ABOUTME: Checks old VS Code settings cannot activate a removed IDE route.
// ABOUTME: Keeps px0 configuration usable beside stale user settings.
use super::*;

#[test]
fn old_ide_settings_are_unknown_and_px0_address_remains() -> anyhow::Result<()> {
    let config = Config::parse(
        "[code]\nmode = 'start'\nlicense_accepted = true\n[keybindings]\nopen_code = 'cmd-alt-c'\n[px0]\nurl = 'http://localhost:7777/'",
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
    assert_eq!(
        config.px0.url.as_ref().map(crate::browser::WebUrl::as_str),
        Some("http://localhost:7777/")
    );
    Ok(())
}
