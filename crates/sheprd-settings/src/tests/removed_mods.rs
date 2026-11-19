use super::*;

#[test]
fn obsolete_mod_settings_are_ignored_while_terminal_font_size_remains() -> anyhow::Result<()> {
    let config = Config::parse("[mods]\nenabled = false\n[terminal]\nsize = 16")?;
    assert!(
        config
            .unknown_keys
            .iter()
            .any(|key| key.starts_with("mods"))
    );
    assert_eq!(config.terminal.size, 16.0);
    Ok(())
}
